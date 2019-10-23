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

use super::{NewPage, NewRevision, PageId, RevisionId, UpdateRevision};
use crate::revision::{CommitInfo, RevisionStore};
use crate::schema::{pages, revisions};
use crate::service_prelude::*;
use crate::user::{User, UserId};
use crate::wiki::WikiId;
use serde_json as json;
use std::collections::HashMap;

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

    pub fn create<S, B>(
        &self,
        slug: &str,
        content: &[u8],
        message: &str,
        wiki_id: WikiId,
        user: &User,
        title: &str,
        alt_title: Option<&str>,
    ) -> Result<(PageId, RevisionId)> {
        #[derive(Debug, Serialize)]
        struct CommitMessage {
            wiki_id: WikiId,
            page_id: PageId,
            user_id: UserId,
        }

        info!("Starting transaction for page creation");

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
            let message = CommitMessage {
                wiki_id,
                page_id,
                user_id,
            };
            let message = json::to_string(&message)?;
            let info = CommitInfo {
                username: user.name(),
                message: &message,
            };

            let store = match self.stores.get(&wiki_id) {
                Some(store) => store,
                None => {
                    error!("No revision store found for wiki ID {}", wiki_id);

                    return Err(Error::StaticMsg("missing revision store for wiki"));
                }
            };

            trace!("Committing content to repository");
            let hash = store.commit(slug, content, info)?;

            let model = NewRevision {
                page_id: page_id.into(),
                user_id: user_id.into(),
                message: &message,
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
}

impl Debug for PageService<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PageService")
            .field("conn", &"PgConnection { .. }")
            .field("stores", &self.stores)
            .finish()
    }
}
