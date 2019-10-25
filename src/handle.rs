/*
 * handle.rs
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

use crate::page::PageService;
use crate::prelude::*;
use crate::user::UserService;
use crate::wiki::WikiService;
use diesel::{Connection, PgConnection};

pub struct Handle {
    conn: Arc<PgConnection>,
    page: PageService,
    user: UserService,
    wiki: WikiService,
}

impl Handle {
    pub fn create(database_url: &str) -> Result<Self> {
        info!("Creating diesel::Handle, establishing connection to Postgres");

        let conn = match PgConnection::establish(database_url) {
            Ok(conn) => Arc::new(conn),
            Err(error) => {
                error!("Error establishing Postgres connection: {}", error);

                return Err(Error::DatabaseConnection(error));
            }
        };

        let page = PageService::new(&conn);
        let user = UserService::new(&conn);
        let wiki = WikiService::new(&conn)?;

        Ok(Handle {
            conn,
            page,
            user,
            wiki,
        })
    }
}

impl Debug for Handle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("diesel::Handle")
            .field("conn", &"PgConnection { .. }")
            .field("page", &self.page)
            .field("user", &self.user)
            .field("wiki", &self.wiki)
            .finish()
    }
}
