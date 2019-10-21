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
use super::{Wiki, WikiId};
use crate::service_prelude::*;

pub struct WikiService<'d> {
    conn: &'d PgConnection,
    tenants: HashMap<WikiId, Wiki>,
}

impl<'d> WikiService<'d> {
    #[inline]
    pub fn new(conn: &'d PgConnection) -> Self {
        WikiService {
            conn,
            tenants: HashMap::new(),
        }
    }

    pub fn create(&mut self, name: &str, slug: &str) -> Result<()> {
        info!("Creating new wiki with name '{}' ('{}')", name, slug);

        let model = NewWiki { name, slug };
        let wiki = diesel::insert_into(wikis::table)
            .values(&model)
            .get_result::<Wiki>(self.conn)?;

        self.tenants.insert(wiki.id(), wiki);

        Ok(())
    }

    pub fn edit(&mut self, id: WikiId, name: Option<&str>, slug: Option<&str>) -> Result<()> {
        use self::wikis::dsl;

        let id: i64 = id.into();
        info!("Editing wiki id {}, name: {:?}, slug: {:?}", id, name, slug);

        let model = UpdateWiki { name, slug };
        diesel::update(dsl::wikis.filter(dsl::wiki_id.eq(id)))
            .set(&model)
            .execute(self.conn)?;

        Ok(())
    }

    pub fn remove(&mut self, id: WikiId) -> Result<()> {
        use self::wikis::dsl;

        // TODO delete all other rows that are FK'ed

        let id: i64 = id.into();
        info!("Removing wiki id {}", id);

        diesel::delete(dsl::wikis.filter(dsl::wiki_id.eq(id))).execute(self.conn)?;

        Ok(())
    }
}

impl Debug for WikiService<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("WikiService")
            .field("conn", &"PgConnection { .. }")
            .field("tenants", &self.tenants)
            .finish()
    }
}
