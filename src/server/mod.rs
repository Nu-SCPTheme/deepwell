/*
 * server/mod.rs
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

use crate::author::AuthorService;
use crate::page::PageService;
use crate::password::PasswordService;
use crate::prelude::*;
use crate::rating::RatingService;
use crate::session::SessionService;
use crate::user::UserService;
use crate::wiki::WikiService;
use diesel::{Connection, PgConnection};
use std::fmt::{self, Debug};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub use self::utils::{normalize_slug, to_lowercase};

#[derive(Debug, Clone)]
pub struct ServerConfig<'a> {
    pub database_url: &'a str,
    pub revisions_dir: PathBuf,
    pub password_blacklist: Option<&'a Path>,
}

pub struct Server {
    conn: Arc<PgConnection>,
    author: AuthorService,
    page: PageService,
    password: PasswordService,
    rating: RatingService,
    session: SessionService,
    user: UserService,
    wiki: WikiService,
}

impl Server {
    pub fn new(config: ServerConfig) -> Result<Self> {
        info!("Creating diesel::Handle, establishing connection to Postgres");

        let ServerConfig {
            database_url,
            revisions_dir,
            password_blacklist,
        } = config;

        let conn = match PgConnection::establish(database_url) {
            Ok(conn) => Arc::new(conn),
            Err(error) => {
                error!("Error establishing Postgres connection: {}", error);

                return Err(Error::DatabaseConnection(error));
            }
        };

        let author = AuthorService::new(&conn);
        let page = PageService::new(&conn, revisions_dir);
        let password = PasswordService::new(&conn, password_blacklist)?;
        let rating = RatingService::new(&conn);
        let session = SessionService::new(&conn);
        let user = UserService::new(&conn);
        let wiki = WikiService::new(&conn)?;

        Ok(Server {
            author,
            conn,
            page,
            password,
            rating,
            session,
            user,
            wiki,
        })
    }

    #[cfg(test)]
    #[inline]
    pub fn test_transaction<F: FnOnce() -> Result<()>>(&self, f: F) {
        self.conn.test_transaction::<_, Error, _>(f);
    }

    /* Helper methods */

    #[inline]
    pub fn transaction<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        self.conn.transaction(f)
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

mod authentication;
mod author;
mod page;
mod rating;
mod revision;
mod session;
mod user;
mod utils;
mod wiki;
