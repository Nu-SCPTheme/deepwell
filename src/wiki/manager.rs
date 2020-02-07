/*
 * wiki/manager.rs
 *
 * deepwell - Database management and migrations service
 * Copyright (C) 2019-2020 Ammon Smith
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

use super::models::{NewWiki, NewWikiSettings, UpdateWiki};
use crate::manager_prelude::*;
use crate::schema::{wiki_settings, wikis};
use async_std::sync::RwLockWriteGuard;

pub struct WikiManager {
    conn: Arc<PgConnection>,
    wikis: RwLock<HashMap<WikiId, Wiki>>,
}

impl WikiManager {
    pub fn new(conn: &Arc<PgConnection>) -> Result<Self> {
        debug!("Creating wiki-manager service");

        let conn = Arc::clone(conn);
        let values = wikis::table.load::<Wiki>(&*conn)?;

        let wikis = {
            let mut map = HashMap::with_capacity(values.len());
            for wiki in values {
                map.insert(wiki.id(), wiki);
            }

            RwLock::new(map)
        };

        Ok(WikiManager { conn, wikis })
    }

    pub async fn create(
        &self,
        name: &str,
        slug: &str,
        domain: &str,
    ) -> Result<(WikiId, RwLockWriteGuard<'_, HashMap<WikiId, Wiki>>)> {
        const DEFAULT_PAGE_LOCK_DURATION: i16 = 900;

        info!("Creating new wiki with name '{}' ('{}')", name, slug);

        self.transaction(async {
            // Insert wiki
            let model = NewWiki { name, slug, domain };
            let wiki = diesel::insert_into(wikis::table)
                .values(&model)
                .get_result::<Wiki>(&*self.conn)?;

            let wiki_id = wiki.id();
            let mut guard = self.wikis.write().await;
            guard.insert(wiki_id, wiki);

            // Insert default wiki settings
            let model = NewWikiSettings {
                wiki_id: wiki_id.into(),
                page_lock_duration: DEFAULT_PAGE_LOCK_DURATION,
            };

            diesel::insert_into(wiki_settings::table)
                .values(&model)
                .execute(&*self.conn)?;

            Ok((wiki_id, guard))
        })
        .await
    }

    pub async fn get_by_id(&self, id: WikiId) -> Result<Wiki> {
        let guard = self.wikis.read().await;
        match guard.get(&id) {
            Some(wiki) => Ok(wiki.clone()),
            None => Err(Error::WikiNotFound),
        }
    }

    pub async fn get_by_slug(&self, slug: &str) -> Result<Wiki> {
        fn get<'a>(wikis: &'a HashMap<WikiId, Wiki>, slug: &'_ str) -> Option<&'a Wiki> {
            for wiki in wikis.values() {
                if wiki.slug() == slug {
                    return Some(wiki);
                }
            }

            None
        }

        let guard = self.wikis.read().await;
        match get(&*guard, slug) {
            Some(wiki) => Ok(wiki.clone()),
            None => Err(Error::WikiNotFound),
        }
    }

    pub async fn get_settings(&self, wiki_id: WikiId) -> Result<WikiSettings> {
        info!("Getting settings for wiki ID {}", wiki_id);

        let id: i64 = wiki_id.into();
        let result = wiki_settings::table
            .find(id)
            .first::<WikiSettings>(&*self.conn)
            .optional()?;

        match result {
            Some(settings) => Ok(settings),
            None => Err(Error::WikiNotFound),
        }
    }

    pub async fn edit(&self, id: WikiId, name: Option<&str>, domain: Option<&str>) -> Result<()> {
        use self::wikis::dsl;

        let model = UpdateWiki { name, domain };

        info!("Editing wiki ID {}: {:?}", id, model);

        if model.has_changes() {
            let id: i64 = id.into();
            diesel::update(dsl::wikis.filter(dsl::wiki_id.eq(id)))
                .set(&model)
                .execute(&*self.conn)?;
        }

        Ok(())
    }
}

impl_async_transaction!(WikiManager);

impl Debug for WikiManager {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("WikiManager")
            .field("conn", &"PgConnection { .. }")
            .field("wikis", &self.wikis)
            .finish()
    }
}
