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

use super::{ChangeType, NewPage, NewRevision, NewTagChange, UpdatePage};
use crate::revision::{CommitInfo, GitHash, RevisionStore};
use crate::schema::{pages, revisions, tag_history};
use crate::service_prelude::*;
use crate::user::{User, UserId};
use crate::wiki::{Wiki, WikiId};
use either::*;
use serde_json as json;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

mod page_id {
    make_id_type!(PageId);
}

mod revision_id {
    make_id_type!(RevisionId);
}

pub use self::page_id::PageId;
pub use self::revision_id::RevisionId;

#[derive(Serialize, Deserialize, Queryable, Debug, Clone, PartialEq, Eq)]
pub struct Page {
    page_id: PageId,
    wiki_id: WikiId,
    slug: String,
    title: String,
    alt_title: Option<String>,
    tags: Vec<String>,
    created_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
}

impl Page {
    #[inline]
    pub fn id(&self) -> PageId {
        self.page_id
    }

    #[inline]
    pub fn wiki_id(&self) -> WikiId {
        self.wiki_id
    }

    #[inline]
    pub fn slug(&self) -> &str {
        &self.slug
    }

    #[inline]
    pub fn title(&self) -> &str {
        &self.title
    }

    #[inline]
    pub fn alt_title(&self) -> Option<&str> {
        self.alt_title.as_ref().map(|s| s as _)
    }

    #[inline]
    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    #[inline]
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    #[inline]
    pub fn deleted_at(&self) -> Option<DateTime<Utc>> {
        self.deleted_at
    }

    #[inline]
    pub fn exists(&self) -> bool {
        self.deleted_at.is_none()
    }
}

pub struct PageService {
    conn: Arc<PgConnection>,
    directory: PathBuf,
    stores: RwLock<HashMap<WikiId, RevisionStore>>,
}

impl PageService {
    #[inline]
    pub fn new(conn: &Arc<PgConnection>, directory: PathBuf) -> Self {
        let conn = Arc::clone(conn);

        PageService {
            conn,
            directory,
            stores: RwLock::new(HashMap::new()),
        }
    }

    fn commit_data(
        &self,
        wiki_id: WikiId,
        page_id: PageId,
        user_id: UserId,
        change_type: ChangeType,
    ) -> Result<String> {
        #[derive(Debug, Serialize)]
        struct CommitMessage {
            wiki_id: WikiId,
            page_id: PageId,
            user_id: UserId,
            change_type: &'static str,
        }

        let message = CommitMessage {
            wiki_id,
            page_id,
            user_id,
            change_type: change_type.into(),
        };

        json::to_string(&message).map_err(Error::from)
    }

    pub fn add_store(&self, wiki: &Wiki) -> Result<()> {
        let repo = self.directory.join(wiki.slug());
        fs::create_dir(&repo)?;

        let store = RevisionStore::new(repo, wiki.domain());
        store.initial_commit()?;

        let mut guard = self.stores.write();
        guard.insert(wiki.id(), store);

        Ok(())
    }

    fn get_store<F, T>(&self, wiki_id: WikiId, f: F) -> Result<T>
    where
        F: FnOnce(&RevisionStore) -> Result<T>,
    {
        trace!("Getting revision store for wiki ID {}", wiki_id);

        let guard = self.stores.read();
        let store = match guard.get(&wiki_id) {
            Some(store) => store,
            None => {
                error!("No revision store found for wiki ID {}", wiki_id);

                return Err(Error::WikiNotFound);
            }
        };

        f(store)
    }

    fn raw_commit(
        &self,
        wiki_id: WikiId,
        slug: &str,
        content: Option<&[u8]>,
        info: CommitInfo,
    ) -> Result<GitHash> {
        self.get_store::<_, GitHash>(wiki_id, |store| {
            trace!("Committing content to repository");
            store.commit(slug, content, info)
        })
    }

    pub fn get_page_id(&self, wiki_id: WikiId, slug: &str) -> Result<Option<PageId>> {
        debug!("Getting page id in wiki ID {} for slug '{}'", wiki_id, slug);

        let wiki_id: i64 = wiki_id.into();
        let page_id = pages::table
            .filter(pages::dsl::wiki_id.eq(wiki_id))
            .filter(pages::dsl::slug.eq(slug))
            .select(pages::dsl::page_id)
            .first::<PageId>(&*self.conn)
            .optional()?;

        Ok(page_id)
    }

    pub fn create(
        &self,
        wiki_id: WikiId,
        slug: &str,
        content: &[u8],
        message: &str,
        user: &User,
        title: &str,
        alt_title: Option<&str>,
    ) -> Result<(PageId, RevisionId)> {
        info!("Starting transaction for page creation");

        self.conn.transaction::<_, Error, _>(|| {
            let model = NewPage {
                wiki_id: wiki_id.into(),
                slug,
                title,
                alt_title,
            };

            trace!("Checking for existing page");
            if self.get_page_id(wiki_id, slug)?.is_some() {
                return Err(Error::PageExists);
            }

            trace!("Inserting {:?} into pages table", &model);
            let page_id = diesel::insert_into(pages::table)
                .values(&model)
                .returning(pages::dsl::page_id)
                .get_result::<PageId>(&*self.conn)?;

            let user_id = user.id();
            let change_type = ChangeType::Create;

            let commit = self.commit_data(wiki_id, page_id, user_id, change_type)?;
            let info = CommitInfo {
                username: user.name(),
                message: &commit,
            };

            let hash = self.raw_commit(wiki_id, slug, Some(content), info)?;
            let model = NewRevision {
                page_id: page_id.into(),
                user_id: user_id.into(),
                message,
                git_commit: hash.as_ref(),
                change_type: change_type.into(),
            };

            trace!("Inserting revision {:?} into revisions table", &model);
            let revision_id = diesel::insert_into(revisions::table)
                .values(&model)
                .returning(revisions::dsl::revision_id)
                .get_result::<RevisionId>(&*self.conn)?;

            Ok((page_id, revision_id))
        })
    }

    pub fn commit(
        &self,
        wiki_id: WikiId,
        slug: &str,
        content: Option<&[u8]>,
        message: &str,
        user: &User,
        title: Option<&str>,
        alt_title: Option<Nullable<&str>>,
    ) -> Result<RevisionId> {
        info!("Starting transaction for page commit");

        self.conn.transaction::<_, Error, _>(|| {
            let model = UpdatePage {
                slug: None,
                title,
                alt_title,
            };

            let page_id = self
                .get_page_id(wiki_id, slug)?
                .ok_or(Error::PageNotFound)?;

            trace!("Updating {:?} in pages table", &model);
            {
                use self::pages::dsl;

                let id: i64 = page_id.into();
                diesel::update(dsl::pages.filter(dsl::page_id.eq(id)))
                    .set(&model)
                    .execute(&*self.conn)?;
            }

            let user_id = user.id();
            let change_type = ChangeType::Modify;

            let commit = self.commit_data(wiki_id, page_id, user_id, change_type)?;
            let info = CommitInfo {
                username: user.name(),
                message: &commit,
            };

            let hash = self.raw_commit(wiki_id, slug, content, info)?;
            let model = NewRevision {
                page_id: page_id.into(),
                user_id: user_id.into(),
                message,
                git_commit: hash.as_ref(),
                change_type: change_type.into(),
            };

            trace!("Inserting revision {:?} into revisions table", &model);
            let revision_id = diesel::insert_into(revisions::table)
                .values(&model)
                .returning(revisions::dsl::revision_id)
                .get_result::<RevisionId>(&*self.conn)?;

            Ok(revision_id)
        })
    }

    pub fn rename(
        &self,
        wiki_id: WikiId,
        old_slug: &str,
        new_slug: &str,
        message: &str,
        user: &User,
    ) -> Result<RevisionId> {
        info!("Starting transaction for page rename");

        self.conn.transaction::<_, Error, _>(|| {
            let model = UpdatePage {
                slug: Some(new_slug),
                title: None,
                alt_title: None,
            };

            let page_id = self
                .get_page_id(wiki_id, old_slug)?
                .ok_or(Error::PageNotFound)?;

            trace!("Updating {:?} in pages table", &model);
            {
                use self::pages::dsl;

                let id: i64 = page_id.into();
                diesel::update(dsl::pages.filter(dsl::page_id.eq(id)))
                    .set(&model)
                    .execute(&*self.conn)?;
            }

            let user_id = user.id();
            let change_type = ChangeType::Rename;

            let commit = self.commit_data(wiki_id, page_id, user_id, change_type)?;
            let info = CommitInfo {
                username: user.name(),
                message: &commit,
            };

            let hash = self.get_store::<_, GitHash>(wiki_id, |store| {
                trace!("Committing rename to repository");
                store.rename(old_slug, new_slug, info)
            })?;

            let model = NewRevision {
                page_id: page_id.into(),
                user_id: user_id.into(),
                message,
                git_commit: hash.as_ref(),
                change_type: change_type.into(),
            };

            trace!("Inserting revision {:?} into revisions table", &model);
            let revision_id = diesel::insert_into(revisions::table)
                .values(&model)
                .returning(revisions::dsl::revision_id)
                .get_result::<RevisionId>(&*self.conn)?;

            Ok(revision_id)
        })
    }

    pub fn remove(
        &self,
        wiki_id: WikiId,
        slug: &str,
        message: &str,
        user: &User,
    ) -> Result<RevisionId> {
        info!("Starting transaction for page removal");

        self.conn.transaction::<_, Error, _>(|| {
            use diesel::dsl::now;

            let page_id = self
                .get_page_id(wiki_id, slug)?
                .ok_or(Error::PageNotFound)?;

            trace!("Marking page as deleted in table");
            {
                use self::pages::dsl;

                let id: i64 = page_id.into();
                diesel::update(dsl::pages.filter(dsl::page_id.eq(id)))
                    .set(pages::dsl::deleted_at.eq(now))
                    .execute(&*self.conn)?;
            }

            let user_id = user.id();
            let change_type = ChangeType::Delete;

            let commit = self.commit_data(wiki_id, page_id, user_id, change_type)?;
            let info = CommitInfo {
                username: user.name(),
                message: &commit,
            };

            let hash = self.get_store::<_, GitHash>(wiki_id, |store| {
                trace!("Committing removal to repository");
                match store.remove(slug, info)? {
                    Some(hash) => Ok(hash),
                    None => Err(Error::PageNotFound),
                }
            })?;

            let model = NewRevision {
                page_id: page_id.into(),
                user_id: user_id.into(),
                message,
                git_commit: hash.as_ref(),
                change_type: change_type.into(),
            };

            trace!("Inserting revision {:?} into revisions table", &model);
            let revision_id = diesel::insert_into(revisions::table)
                .values(&model)
                .returning(revisions::dsl::revision_id)
                .get_result::<RevisionId>(&*self.conn)?;

            Ok(revision_id)
        })
    }

    pub fn tags(
        &self,
        wiki_id: WikiId,
        slug: &str,
        message: &str,
        user: &User,
        tags: &mut [&str],
    ) -> Result<RevisionId> {
        info!("Starting transaction for page tags");

        self.conn.transaction::<_, Error, _>(|| {
            let page_id = self
                .get_page_id(wiki_id, slug)?
                .ok_or(Error::PageNotFound)?;

            trace!("Getting tag difference");
            let current_tags = {
                let id: i64 = page_id.into();
                pages::table
                    .find(id)
                    .select(pages::dsl::tags)
                    .first::<Vec<String>>(&*self.conn)?
            };

            let (added_tags, removed_tags) = tag_diff(&current_tags, tags);

            // Create commit
            let user_id = user.id();
            let change_type = ChangeType::Tags;

            let commit = self.commit_data(wiki_id, page_id, user_id, change_type)?;
            let info = CommitInfo {
                username: user.name(),
                message: &commit,
            };

            let hash = self.get_store::<_, GitHash>(wiki_id, |store| {
                trace!("Committing tag changes to repository");
                store.empty_commit(info)
            })?;

            let model = NewRevision {
                page_id: page_id.into(),
                user_id: user_id.into(),
                message,
                git_commit: hash.as_ref(),
                change_type: change_type.into(),
            };

            trace!("Inserting revision {:?} into revisions table", &model);
            let revision_id = diesel::insert_into(revisions::table)
                .values(&model)
                .returning(revisions::dsl::revision_id)
                .get_result::<RevisionId>(&*self.conn)?;

            let model = NewTagChange {
                revision_id: revision_id.into(),
                added_tags: &added_tags,
                removed_tags: &removed_tags,
            };

            trace!("Inserting tag change {:?} into tag history table", &model);
            diesel::insert_into(tag_history::table)
                .values(&model)
                .execute(&*self.conn)?;

            tags.sort();

            trace!("Updating tags for page");
            diesel::update(pages::table)
                .set(pages::dsl::tags.eq(&*tags))
                .execute(&*self.conn)?;

            Ok(revision_id)
        })
    }

    pub fn check_page(&self, wiki_id: WikiId, slug: &str) -> Result<bool> {
        info!(
            "Checking if page for exists in wiki ID {}, slug {} exists",
            wiki_id, slug,
        );

        let result = pages::table
            .filter(pages::slug.eq(slug))
            .filter(pages::deleted_at.is_null())
            .select(pages::page_id)
            .first::<PageId>(&*self.conn)
            .optional()?;

        Ok(result.is_some())
    }

    pub fn get_page(&self, wiki_id: WikiId, slug: &str) -> Result<Option<Page>> {
        info!("Getting page for wiki ID {}, slug {}", wiki_id, slug);

        let wiki_id: i64 = wiki_id.into();
        let page = pages::table
            .filter(pages::wiki_id.eq(wiki_id))
            .filter(pages::slug.eq(slug))
            .filter(pages::deleted_at.is_null())
            .first::<Page>(&*self.conn)
            .optional()?;

        Ok(page)
    }

    pub fn get_page_contents(&self, wiki_id: WikiId, slug: &str) -> Result<Option<Box<[u8]>>> {
        info!("Getting page for wiki ID {}, slug {}", wiki_id, slug);

        self.get_store(wiki_id, |store| store.get_page(slug))
    }

    pub fn get_blame(&self, wiki_id: WikiId, slug: &str) -> Result<Option<Blame>> {
        info!("Getting blame for wiki ID {}, slug {}", wiki_id, slug);

        self.get_store(wiki_id, |store| store.get_blame(slug))
    }

    fn commit_hash(&self, spec: Either<RevisionId, GitHash>) -> Result<GitHash> {
        let id: i64 = match spec {
            Left(id) => id.into(),
            Right(hash) => return Ok(hash),
        };

        debug!("Getting commit hash for revision {}", id);

        let result = revisions::table
            .find(id)
            .select(revisions::dsl::git_commit)
            .first::<Vec<u8>>(&*self.conn)
            .optional()?;

        match result {
            Some(hash) => Ok(GitHash::from(hash.as_slice())),
            None => Err(Error::RevisionNotFound),
        }
    }

    pub fn get_page_version(
        &self,
        wiki_id: WikiId,
        slug: &str,
        revision: Either<RevisionId, GitHash>,
    ) -> Result<Option<Box<[u8]>>> {
        info!(
            "Getting specific page version for wiki ID {}, slug {}",
            wiki_id, slug
        );

        let hash = self.commit_hash(revision)?;

        self.get_store(wiki_id, |store| store.get_page_version(slug, hash))
    }

    pub fn get_diff(
        &self,
        wiki_id: WikiId,
        slug: &str,
        first: Either<RevisionId, GitHash>,
        second: Either<RevisionId, GitHash>,
    ) -> Result<Box<[u8]>> {
        info!("Getting diff for wiki ID {}, slug {}", wiki_id, slug);

        let first = self.commit_hash(first)?;
        let second = self.commit_hash(second)?;

        self.get_store(wiki_id, |store| store.get_diff(slug, first, second))
    }

    pub fn edit_revision(&self, revision_id: RevisionId, message: &str) -> Result<()> {
        use self::revisions::dsl;

        info!("Editing revision message for ID {}", revision_id);

        let id: i64 = revision_id.into();
        diesel::update(dsl::revisions.filter(dsl::revision_id.eq(id)))
            .set(dsl::message.eq(message))
            .execute(&*self.conn)?;

        Ok(())
    }

    pub fn set_domain(&self, wiki_id: WikiId, new_domain: &str) -> Result<()> {
        self.get_store(wiki_id, |store| {
            store.set_domain(new_domain);
            Ok(())
        })
    }
}

impl Debug for PageService {
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
    let old_tags = current_tags
        .iter()
        .map(|s| s.as_str())
        .collect::<HashSet<_>>();

    let added_tags = difference!(new_tags, old_tags);
    let removed_tags = difference!(old_tags, new_tags);

    (added_tags, removed_tags)
}
