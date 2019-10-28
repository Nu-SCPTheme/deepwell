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
use crate::rating::{RatingHistory, RatingId, RatingService};
use crate::user::UserService;
use crate::wiki::{UpdateWiki, WikiService};
use diesel::{Connection, PgConnection};
use either::*;
use std::fmt::{self, Debug};
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct ServerConfig<'a> {
    pub database_url: &'a str,
    pub revisions_dir: PathBuf,
}

pub struct Server {
    conn: Rc<PgConnection>,
    page: PageService,
    rating: RatingService,
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
            Ok(conn) => Rc::new(conn),
            Err(error) => {
                error!("Error establishing Postgres connection: {}", error);

                return Err(Error::DatabaseConnection(error));
            }
        };

        let page = PageService::new(&conn, revisions_dir);
        let rating = RatingService::new(&conn);
        let user = UserService::new(&conn);
        let wiki = WikiService::new(&conn)?;

        Ok(Server {
            conn,
            page,
            rating,
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

            self.page.add_store(&wiki)?;
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

    /// Creates a new page with the given contents and metadata.
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

    /// Edits an existing page to have the given content.
    /// Optionally permits modifying the title or alternate title.
    /// (An empty alternate title signifies that none is used)
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

    /// Renames a page to use a different slug.
    pub fn rename_page(
        &self,
        old_slug: &str,
        new_slug: &str,
        message: &str,
        wiki_id: WikiId,
        page_id: PageId,
        user: Either<UserId, &User>,
    ) -> Result<RevisionId> {
        let mut user_obj = None;
        let user = self.get_user(user, &mut user_obj)?;

        self.page
            .rename(old_slug, new_slug, message, wiki_id, page_id, user)
    }

    /// Removes the given page.
    pub fn remove_page(
        &self,
        slug: &str,
        message: &str,
        wiki_id: WikiId,
        page_id: PageId,
        user: Either<UserId, &User>,
    ) -> Result<RevisionId> {
        let mut user_obj = None;
        let user = self.get_user(user, &mut user_obj)?;

        self.page.remove(slug, message, wiki_id, page_id, user)
    }

    /// Determines if a page with the given slug exists.
    #[inline]
    pub fn check_page(&self, wiki_id: WikiId, slug: &str) -> Result<bool> {
        self.page.check_page(wiki_id, slug)
    }

    /// Gets the metadata for a given page, as well as its rating information.
    /// Uses Wikidot's `ups - downs` formula for scoring.
    pub fn get_page(&self, wiki_id: WikiId, slug: &str) -> Result<Option<(Page, Rating)>> {
        debug!("Creating transaction for page and rating");

        self.conn.transaction::<_, Error, _>(|| {
            let page = match self.page.get_page(wiki_id, slug)? {
                Some(page) => page,
                None => return Ok(None),
            };
            let rating = self.rating.get_rating(page.id())?;

            Ok(Some((page, rating)))
        })
    }

    /// Gets the contents for a given page, as well as its page ID.
    pub fn get_page_contents(
        &self,
        wiki_id: WikiId,
        slug: &str,
    ) -> Result<Option<(PageId, Box<[u8]>)>> {
        self.page.get_page_contents(wiki_id, slug)
    }

    /// Sets all the tags for a given page.
    pub fn set_page_tags(
        &self,
        message: &str,
        wiki_id: WikiId,
        page_id: PageId,
        user: Either<UserId, &User>,
        tags: &[&str],
    ) -> Result<RevisionId> {
        let mut user_obj = None;
        let user = self.get_user(user, &mut user_obj)?;

        self.page.tags(message, wiki_id, page_id, user, tags)
    }

    /* Rating methods */

    /// Sets the rating for a given page and user.
    #[inline]
    pub fn set_rating(&self, page_id: PageId, user_id: UserId, rating: i16) -> Result<RatingId> {
        self.rating.add(page_id, user_id, rating)
    }

    /// Gets all changes in the rating for a given page and user.
    /// Earliest entries appear near the beginning.
    #[inline]
    pub fn get_rating_history(
        &self,
        page_id: PageId,
        user_id: UserId,
    ) -> Result<Vec<RatingHistory>> {
        self.rating.get_history(page_id, user_id)
    }

    /// Gets the latest rating history entry for the given page and user.
    #[inline]
    pub fn get_rating_history_entry_last(
        &self,
        page_id: PageId,
        user_id: UserId,
    ) -> Result<Option<RatingHistory>> {
        self.rating.get_history_latest(page_id, user_id)
    }

    /// Gets the rating history entry with the given ID, if it exists.
    #[inline]
    pub fn get_rating_history_entry_from_id(
        &self,
        rating_id: RatingId,
    ) -> Result<Option<RatingHistory>> {
        self.rating.get_history_entry(rating_id)
    }

    /* Revision methods */

    /// Get the version of a page at the specified revision.
    #[inline]
    pub fn get_page_version(
        &self,
        wiki_id: WikiId,
        slug: &str,
        revision: Either<RevisionId, GitHash>,
    ) -> Result<Option<Box<[u8]>>> {
        self.page.get_page_version(wiki_id, slug, revision)
    }

    /// Get the blame for a given page, if it exists.
    #[inline]
    pub fn get_page_blame(&self, wiki_id: WikiId, slug: &str) -> Result<Option<Blame>> {
        self.page.get_blame(wiki_id, slug)
    }

    /// Get a diff for a given page between the two specified revisions.
    #[inline]
    pub fn get_page_diff(
        &self,
        wiki_id: WikiId,
        slug: &str,
        first: Either<RevisionId, GitHash>,
        second: Either<RevisionId, GitHash>,
    ) -> Result<Box<[u8]>> {
        self.page.get_diff(wiki_id, slug, first, second)
    }

    /// Overwrite the revision message for a given change.
    #[inline]
    pub fn edit_revision(&self, revision_id: RevisionId, message: &str) -> Result<()> {
        self.page.edit_revision(revision_id, message)
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
