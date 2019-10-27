/*
 * server.rs
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
use std::fmt::{self, Debug};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ServerConfig<'a> {
    pub database_url: &'a str,
    pub revisions_dir: PathBuf,
}

pub struct Server {
    conn: Arc<PgConnection>,
    page: PageService,
    user: UserService,
    wiki: WikiService,
}

impl Server {
    pub fn new(config: ServerConfig) -> Result<Self> {
        info!("Creating diesel::Handle, establishing connection to Postgres");

        let ServerConfig {
            database_url,
            revisions_dir,
        } = config;

        let conn = match PgConnection::establish(database_url) {
            Ok(conn) => Arc::new(conn),
            Err(error) => {
                error!("Error establishing Postgres connection: {}", error);

                return Err(Error::DatabaseConnection(error));
            }
        };

        let page = PageService::new(&conn, revisions_dir);
        let user = UserService::new(&conn);
        let wiki = WikiService::new(&conn)?;

        Ok(Server {
            conn,
            page,
            user,
            wiki,
        })
    }

    /// Creates a new Wiki with the given parameters. Returns the ID of the created instance.
    pub fn create_wiki(&self, name: &str, slug: &str, domain: &str) -> Result<WikiId> {
        let id = self.wiki.create(name, slug, domain)?;
        self.wiki.get_by_id(id, |wiki| {
            let wiki = wiki.expect("Can't find wiki object after inserting");

            self.page.add_store(&wiki);
            Ok(id)
        })
    }
}

impl Debug for Server {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("diesel::Handle")
            .field("conn", &"PgConnection { .. }")
            .field("page", &self.page)
            .field("user", &self.user)
            .field("wiki", &self.wiki)
            .finish()
    }
}
