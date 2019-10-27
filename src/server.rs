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
use either::*;
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

    /* Wiki methods */

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

    /* User methods */

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

    /// Get the model for a user from its ID.
    #[inline]
    pub fn get_user_from_id(&self, id: UserId) -> Result<Option<User>> {
        self.user.get_from_id(id)
    }

    /// Gets the models for users from their IDs.
    /// Results are returned in the same order as the IDs, and any missing
    /// users give `None` instead.
    #[inline]
    pub fn get_users_from_ids(&self, ids: &[UserId]) -> Result<Vec<Option<User>>> {
        self.user.get_from_ids(ids)
    }

    /// Gets the model for a user from its name.
    #[inline]
    pub fn get_users_from_name(&self, name: &str) -> Result<Option<User>> {
        self.user.get_from_name(name)
    }

    /// Gets the model for a user from its email.
    #[inline]
    pub fn get_user_from_email(&self, email: &str) -> Result<Option<User>> {
        self.user.get_from_email(email)
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

    /* Page methods */

    fn get_user<'a>(
        &self,
        user: Either<UserId, &'a User>,
        storage: &'a mut Option<User>,
    ) -> Result<&'a User> {
        match user {
            Right(user) => Ok(user),
            Left(id) => match self.user.get_from_id(id) {
                Ok(Some(user)) => {
                    *storage = Some(user);
                    Ok(storage.as_ref().unwrap())
                }
                Ok(None) => Err(Error::UserNotFound),
                Err(error) => Err(error),
            },
        }
    }

    pub fn create_page(
        &self,
        slug: &str,
        content: &[u8],
        message: &str,
        wiki_id: WikiId,
        user: Either<UserId, &User>,
        title: &str,
        alt_title: &str,
    ) -> Result<(PageId, RevisionId)> {
        let mut user_obj = None;
        let user = self.get_user(user, &mut user_obj)?;

        self.page
            .create(slug, content, message, wiki_id, &user, title, alt_title)
    }

    pub fn edit_page(
        &self,
        slug: &str,
        content: &[u8],
        message: &str,
        wiki_id: WikiId,
        page_id: PageId,
        user: Either<UserId, &User>,
        title: Option<&str>,
        alt_title: Option<&str>,
    ) -> Result<RevisionId> {
        let mut user_obj = None;
        let user = self.get_user(user, &mut user_obj)?;

        // Empty string means use default
        let alt_title: Option<Option<&str>> = match alt_title {
            Some("") => Some(None),
            Some(_) => Some(alt_title),
            None => None,
        };

        self.page.commit(
            slug, content, message, wiki_id, page_id, user, title, alt_title,
        )
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
