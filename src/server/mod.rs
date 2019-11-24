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

mod author;
mod lock;
mod page;
mod password;
mod rating;
mod revision;
mod session;
mod user;
mod utils;
mod wiki;

use crate::author::AuthorManager;
use crate::lock::LockManager;
use crate::manager_prelude::*;
use crate::page::PageManager;
use crate::password::PasswordManager;
use crate::rating::RatingManager;
use crate::session::SessionManager;
use crate::user::UserManager;
use crate::wiki::WikiManager;
use std::fmt::{self, Debug};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Config<'a> {
    pub database_url: &'a str,
    pub revisions_dir: PathBuf,
    pub password_blacklist: Option<&'a Path>,
}

pub struct Server {
    conn: Arc<PgConnection>,
    author: AuthorManager,
    lock: LockManager,
    page: PageManager,
    password: PasswordManager,
    rating: RatingManager,
    session: SessionManager,
    user: UserManager,
    wiki: WikiManager,
}

impl Server {
    pub fn new(config: Config) -> Result<Self> {
        info!("Creating deepwell::Server, establishing connection to Postgres");

        let Config {
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

        let author = AuthorManager::new(&conn);
        let lock = LockManager::new(&conn);
        let page = PageManager::new(&conn, revisions_dir);
        let password = PasswordManager::new(&conn, password_blacklist)?;
        let rating = RatingManager::new(&conn);
        let session = SessionManager::new(&conn);
        let user = UserManager::new(&conn);
        let wiki = WikiManager::new(&conn)?;

        Ok(Server {
            conn,
            author,
            lock,
            page,
            password,
            rating,
            session,
            user,
            wiki,
        })
    }

    /* Helper methods */

    #[cfg(test)]
    #[inline]
    pub fn test_transaction<F: FnOnce() -> Result<()>>(&self, f: F) {
        self.conn.test_transaction::<_, Error, _>(f);
    }
}

impl_async_transaction!(Server);

impl Debug for Server {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("deepwell::Server")
            .field("conn", &"PgConnection { .. }")
            .field("page", &self.page)
            .field("user", &self.user)
            .field("wiki", &self.wiki)
            .finish()
    }
}
