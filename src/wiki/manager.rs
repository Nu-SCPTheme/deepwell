/*
 * wiki/manager.rs
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

use super::models::{NewWiki, UpdateWiki};
use crate::manager_prelude::*;
use crate::schema::{wiki_settings, wikis};
use async_std::sync::RwLockWriteGuard;
use std::time::Duration;

make_id_type!(WikiId);

#[derive(Serialize, Deserialize, Queryable, Debug, Clone, PartialEq, Eq)]
pub struct Wiki {
    id: WikiId,
    name: String,
    slug: String,
    domain: String,
    created_at: DateTime<Utc>,
}

impl Wiki {
    #[inline]
    pub fn id(&self) -> WikiId {
        self.id
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn slug(&self) -> &str {
        &self.slug
    }

    #[inline]
    pub fn domain(&self) -> &str {
        &self.domain
    }

    #[inline]
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

#[derive(Serialize, Deserialize, Queryable, Debug, Clone, PartialEq, Eq)]
pub struct WikiSettings {
    id: WikiId,
    page_lock_duration: i16,
}

impl WikiSettings {
    #[inline]
    pub fn id(&self) -> WikiId {
        self.id
    }

    #[inline]
    pub fn page_lock_duration(&self) -> Duration {
        let minutes = self.page_lock_duration as u64;

        Duration::from_secs(minutes * 60)
    }
}

pub struct WikiManager {
    conn: Arc<PgConnection>,
    wikis: RwLock<HashMap<WikiId, Wiki>>,
}

impl WikiManager {
    pub fn new(conn: &Arc<PgConnection>) -> Result<Self> {
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
        info!("Creating new wiki with name '{}' ('{}')", name, slug);

        let model = NewWiki { name, slug, domain };
        let wiki = diesel::insert_into(wikis::table)
            .values(&model)
            .get_result::<Wiki>(&*self.conn)?;

        let wiki_id = wiki.id();
        let mut guard = self.wikis.write().await;
        guard.insert(wiki_id, wiki);

        Ok((wiki_id, guard))
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
