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
use crate::wiki::{UpdateWiki, WikiService};
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

    /// Creates a new wiki with the given parameters. Returns its ID.
    pub fn create_wiki(&self, name: &str, slug: &str, domain: &str) -> Result<WikiId> {
        let id = self.wiki.create(name, slug, domain)?;
        self.wiki.get_by_id(id, |wiki| {
            let wiki = wiki.expect("Can't find wiki object after inserting");

            self.page.add_store(&wiki);
            Ok(id)
        })
    }

    /// Renames the wiki with the given ID.
    /// Changing a wiki's slug is not supported.
    pub fn rename_wiki(&self, id: WikiId, new_name: &str) -> Result<()> {
        let model = UpdateWiki {
            name: Some(new_name),
            domain: None,
        };

        self.wiki.edit(id, model)?;
        Ok(())
    }

    /// Creates a new user with the given name and email. Returns its ID.
    #[inline]
    pub fn create_user(&self, name: &str, email: &str) -> Result<UserId> {
        self.user.create(name, email)
    }

    /// Edits data attached to a user with the given ID.
    #[inline]
    pub fn edit_user(
        &self,
        id: UserId,
        name: Option<&str>,
        email: Option<&str>,
        author_page: Option<&str>,
        website: Option<&str>,
        about: Option<&str>,
        gender: Option<&str>,
        location: Option<&str>,
    ) -> Result<()> {
        self.user.edit(
            id,
            name,
            email,
            author_page,
            website,
            about,
            gender,
            location,
        )
    }

    /// Marks a user as verified.
    #[inline]
    pub fn verify_user(&self, id: UserId) -> Result<()> {
        self.user.verify(id)
    }

    /// Marks the user as "inactive", effectively deleting them.
    #[inline]
    pub fn make_user_inactive(&self, id: UserId) -> Result<()> {
        self.user.mark_inactive(id, true)
    }

    /// Marks the user as "active" again, effectively un-deleting them.
    #[inline]
    pub fn make_user_active(&self, id: UserId) -> Result<()> {
        self.user.mark_inactive(id, false)
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
