/*
 * wiki/service.rs
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
use crate::schema::wikis;
use crate::service_prelude::*;

make_id_type!(WikiId);

#[derive(Serialize, Deserialize, Queryable, Debug, Clone, PartialEq, Eq)]
pub struct Wiki {
    id: WikiId,
    name: String,
    slug: String,
    created_at: NaiveDateTime,
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
    pub fn created_at(&self) -> NaiveDateTime {
        self.created_at
    }
}

pub struct WikiService {
    conn: Arc<PgConnection>,
    wikis: Mutex<HashMap<WikiId, Wiki>>,
}

impl WikiService {
    pub fn new(conn: &Arc<PgConnection>) -> Result<Self> {
        let conn = Arc::clone(conn);
        let values = wikis::table
            .filter(wikis::wiki_id.ge(0))
            .load::<Wiki>(&*conn)?;

        let wikis = {
            let mut map = HashMap::with_capacity(values.len());
            for wiki in values {
                map.insert(wiki.id(), wiki);
            }

            Mutex::new(map)
        };

        Ok(WikiService { conn, wikis })
    }

    pub fn create(&self, name: &str, slug: &str) -> Result<()> {
        info!("Creating new wiki with name '{}' ('{}')", name, slug);

        let model = NewWiki { name, slug };
        diesel::insert_into(wikis::table)
            .values(&model)
            .execute(&*self.conn)?;

        Ok(())
    }

    pub fn get_by_id<F, T>(&self, id: WikiId, f: F) -> Result<T>
    where
        F: FnOnce(Option<&Wiki>) -> Result<T>,
    {
        let guard = self.wikis.lock();
        let wiki = guard.get(&id);
        f(wiki)
    }

    pub fn get_by_slug<F, T>(&self, slug: &str, f: F) -> Result<T>
    where
        F: FnOnce(Option<&Wiki>) -> Result<T>,
    {
        fn get<'a>(wikis: &'a HashMap<WikiId, Wiki>, slug: &'_ str) -> Option<&'a Wiki> {
            for wiki in wikis.values() {
                if wiki.slug() == slug {
                    return Some(wiki);
                }
            }

            None
        }

        let guard = self.wikis.lock();
        let wiki = get(&*guard, slug);
        f(wiki)
    }

    pub fn edit(&self, id: WikiId, name: Option<&str>, slug: Option<&str>) -> Result<()> {
        use self::wikis::dsl;

        if name.is_none() && slug.is_none() {
            warn!("Empty wiki metadata update");
            return Ok(());
        }

        info!("Editing wiki id {}, name: {:?}, slug: {:?}", id, name, slug);

        let id: i64 = id.into();
        let model = UpdateWiki { name, slug };
        diesel::update(dsl::wikis.filter(dsl::wiki_id.eq(id)))
            .set(&model)
            .execute(&*self.conn)?;

        Ok(())
    }
}

impl Debug for WikiService {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("WikiService")
            .field("conn", &"PgConnection { .. }")
            .field("wikis", &self.wikis)
            .finish()
    }
}
