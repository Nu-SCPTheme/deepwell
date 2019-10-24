/*
 * page/service.rs
 *
 * deepwell - Database management and migrations service
 * Copyright (C) 2019 Ammon Smith
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

use super::{NewPage, NewRevision, NewTagChange, UpdatePage, UpdateRevision};
use crate::revision::{CommitInfo, RevisionStore};
use crate::schema::{pages, revisions, tag_history};
use crate::service_prelude::*;
use crate::user::{User, UserId};
use crate::wiki::WikiId;
use serde_json as json;
use std::collections::{HashMap, HashSet};

mod page_id {
    make_id_type!(PageId);
}

mod revision_id {
    make_id_type!(RevisionId);
}

pub use self::page_id::PageId;
pub use self::revision_id::RevisionId;

pub struct PageService<'d> {
    conn: &'d PgConnection,
    stores: HashMap<WikiId, RevisionStore>,
}

impl<'d> PageService<'d> {
    #[inline]
    pub fn new(conn: &'d PgConnection) -> Self {
        PageService {
            conn,
            stores: HashMap::new(),
        }
    }

    fn commit_data(&self, wiki_id: WikiId, page_id: PageId, user_id: UserId) -> Result<String> {
        #[derive(Debug, Serialize)]
        struct CommitMessage {
            wiki_id: WikiId,
            page_id: PageId,
            user_id: UserId,
        }

        let message = CommitMessage {
            wiki_id,
            page_id,
            user_id,
        };

        json::to_string(&message).map_err(Error::from)
    }

    fn get_store(&self, wiki_id: WikiId) -> Result<&RevisionStore> {
        trace!("Getting revision store for wiki ID {}", wiki_id);

        match self.stores.get(&wiki_id) {
            Some(store) => Ok(store),
            None => {
                error!("No revision store found for wiki ID {}", wiki_id);

                return Err(Error::StaticMsg("missing revision store for wiki"));
            }
        }
    }

    pub fn create(
        &self,
        slug: &str,
        content: &[u8],
        message: &str,
        wiki_id: WikiId,
        user: &User,
        title: &str,
        alt_title: &str,
    ) -> Result<(PageId, RevisionId)> {
        info!("Starting transaction for page creation");

        // Empty string means use default
        let alt_title: Option<&str> = match alt_title {
            "" => None,
            _ => Some(alt_title),
        };

        self.conn.transaction::<_, Error, _>(|| {
            let model = NewPage {
                wiki_id: wiki_id.into(),
                slug,
                title,
                alt_title,
            };

            trace!("Inserting {:?} into pages table", &model);
            let page_id = diesel::insert_into(pages::table)
                .values(&model)
                .returning(pages::dsl::page_id)
                .get_result::<PageId>(self.conn)?;

            let user_id = user.id();
            let commit = self.commit_data(wiki_id, page_id, user_id)?;
            let store = self.get_store(wiki_id)?;

            let info = CommitInfo {
                username: user.name(),
                message: &commit,
            };

            trace!("Committing content to repository");
            let hash = store.commit(slug, content, info)?;

            let model = NewRevision {
                page_id: page_id.into(),
                user_id: user_id.into(),
                message,
                git_commit: hash.as_ref(),
                change_type: "create",
            };

            trace!("Inserting revision {:?} into revisions table", &model);
            let revision_id = diesel::insert_into(revisions::table)
                .values(&model)
                .returning(revisions::dsl::revision_id)
                .get_result::<RevisionId>(self.conn)?;

            Ok((page_id, revision_id))
        })
    }

    pub fn commit(
        &self,
        slug: &str,
        content: &[u8],
        message: &str,
        wiki_id: WikiId,
        page_id: PageId,
        user: &User,
        title: Option<&str>,
        alt_title: Option<&str>,
    ) -> Result<RevisionId> {
        info!("Starting transaction for page commit");

        // Empty string means use default
        let alt_title: Option<Option<&str>> = match alt_title {
            Some("") => Some(None),
            Some(_) => Some(alt_title),
            None => None,
        };

        self.conn.transaction::<_, Error, _>(|| {
            let model = UpdatePage {
                slug: None,
                title,
                alt_title,
            };

            trace!("Updating {:?} in pages table", &model);
            {
                use self::pages::dsl;

                let id: i64 = page_id.into();
                diesel::update(dsl::pages.filter(dsl::page_id.eq(id)))
                    .set(&model)
                    .execute(self.conn)?;
            }

            let user_id = user.id();
            let commit = self.commit_data(wiki_id, page_id, user_id)?;
            let store = self.get_store(wiki_id)?;

            let info = CommitInfo {
                username: user.name(),
                message: &commit,
            };

            trace!("Committing content to repository");
            let hash = store.commit(slug, content, info)?;

            let model = NewRevision {
                page_id: page_id.into(),
                user_id: user_id.into(),
                message,
                git_commit: hash.as_ref(),
                change_type: "modify",
            };

            trace!("Inserting revision {:?} into revisions table", &model);
            let revision_id = diesel::insert_into(revisions::table)
                .values(&model)
                .returning(revisions::dsl::revision_id)
                .get_result::<RevisionId>(self.conn)?;

            Ok(revision_id)
        })
    }

    pub fn rename(
        &self,
        old_slug: &str,
        new_slug: &str,
        message: &str,
        wiki_id: WikiId,
        page_id: PageId,
        user: &User,
    ) -> Result<RevisionId> {
        info!("Starting transaction for page rename");

        self.conn.transaction::<_, Error, _>(|| {
            let model = UpdatePage {
                slug: Some(new_slug),
                title: None,
                alt_title: None,
            };

            trace!("Updating {:?} in pages table", &model);
            {
                use self::pages::dsl;

                let id: i64 = page_id.into();
                diesel::update(dsl::pages.filter(dsl::page_id.eq(id)))
                    .set(&model)
                    .execute(self.conn)?;
            }

            let user_id = user.id();
            let commit = self.commit_data(wiki_id, page_id, user_id)?;
            let store = self.get_store(wiki_id)?;

            let info = CommitInfo {
                username: user.name(),
                message: &commit,
            };

            trace!("Committing rename to repository");
            let hash = store.rename(old_slug, new_slug, info)?;

            let model = NewRevision {
                page_id: page_id.into(),
                user_id: user_id.into(),
                message,
                git_commit: hash.as_ref(),
                change_type: "rename",
            };

            trace!("Inserting revision {:?} into revisions table", &model);
            let revision_id = diesel::insert_into(revisions::table)
                .values(&model)
                .returning(revisions::dsl::revision_id)
                .get_result::<RevisionId>(self.conn)?;

            Ok(revision_id)
        })
    }

    pub fn remove(
        &self,
        slug: &str,
        message: &str,
        wiki_id: WikiId,
        page_id: PageId,
        user: &User,
    ) -> Result<RevisionId> {
        info!("Starting transaction for page removal");

        self.conn.transaction::<_, Error, _>(|| {
            use diesel::dsl::now;

            trace!("Marking page as deleted in table");
            {
                use self::pages::dsl;

                let id: i64 = page_id.into();
                diesel::update(dsl::pages.filter(dsl::page_id.eq(id)))
                    .set(pages::dsl::deleted_at.eq(now))
                    .execute(self.conn)?;
            }

            let user_id = user.id();
            let commit = self.commit_data(wiki_id, page_id, user_id)?;
            let store = self.get_store(wiki_id)?;

            let info = CommitInfo {
                username: user.name(),
                message: &commit,
            };

            trace!("Committing removal to repository");
            let hash = match store.remove(slug, info)? {
                Some(hash) => hash,
                None => return Err(Error::PageNotFound),
            };

            let model = NewRevision {
                page_id: page_id.into(),
                user_id: user_id.into(),
                message,
                git_commit: hash.as_ref(),
                change_type: "delete",
            };

            trace!("Inserting revision {:?} into revisions table", &model);
            let revision_id = diesel::insert_into(revisions::table)
                .values(&model)
                .returning(revisions::dsl::revision_id)
                .get_result::<RevisionId>(self.conn)?;

            Ok(revision_id)
        })
    }

    pub fn tags(
        &self,
        slug: &str,
        message: &str,
        wiki_id: WikiId,
        page_id: PageId,
        user: &User,
        tags: &[&str],
    ) -> Result<RevisionId> {
        info!("Starting transaction for page tags");

        self.conn.transaction::<_, Error, _>(|| {
            trace!("Getting tag difference");
            let current_tags = {
                let id: i64 = page_id.into();
                pages::table
                    .find(id)
                    .select(pages::dsl::tags)
                    .first::<Vec<String>>(self.conn)?
            };

            let (added_tags, removed_tags) = tag_diff(&current_tags, tags);

            // Create commit
            let user_id = user.id();
            let commit = self.commit_data(wiki_id, page_id, user_id)?;
            let store = self.get_store(wiki_id)?;

            let info = CommitInfo {
                username: user.name(),
                message: &commit,
            };

            trace!("Committing tag changes to repository");
            let hash = store.empty_commit(info)?;
            let model = NewRevision {
                page_id: page_id.into(),
                user_id: user_id.into(),
                message,
                git_commit: hash.as_ref(),
                change_type: "tags",
            };

            trace!("Inserting revision {:?} into revisions table", &model);
            let revision_id = diesel::insert_into(revisions::table)
                .values(&model)
                .returning(revisions::dsl::revision_id)
                .get_result::<RevisionId>(self.conn)?;

            let model = NewTagChange {
                revision_id: revision_id.into(),
                added_tags: &added_tags,
                removed_tags: &removed_tags,
            };

            trace!("Inserting tag change {:?} into tag history table", &model);
            diesel::insert_into(tag_history)
                .values(&model)
                .execute(self.conn)?;

            Ok(revision_id)
        })
    }

    pub fn edit_revision(&self, revision_id: RevisionId, message: &str) -> Result<()> {
        use self::revisions::dsl;

        info!("Editing revision message for ID {}", revision_id);

        let id: i64 = revision_id.into();
        diesel::update(dsl::revisions.filter(dsl::revision_id.eq(id)))
            .set(dsl::message.eq(message))
            .execute(self.conn)?;

        Ok(())
    }
}

impl Debug for PageService<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PageService")
            .field("conn", &"PgConnection { .. }")
            .field("stores", &self.stores)
            .finish()
    }
}

fn tag_diff<'a>(
    current_tags: &'a [String],
    new_tags: &'_ [&'a str],
) -> (Vec<&'a str>, Vec<&'a str>) {
    macro_rules! difference {
        ($first:expr, $second:expr) => {{
            let mut diff: Vec<_> = $first.difference(&$second).copied().collect();
            diff.sort();
            diff
        }};
    }

    let new_tags = new_tags.iter().copied().collect::<HashSet<_>>();
    let old_tags = new_tags.iter().copied().collect::<HashSet<_>>();

    let added_tags = difference!(new_tags, old_tags);
    let removed_tags = difference!(old_tags, new_tags);

    (added_tags, removed_tags)
}
