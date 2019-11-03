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

use crate::author::{Author, AuthorService, AuthorType};
use crate::page::PageService;
use crate::password::PasswordService;
use crate::prelude::*;
use crate::rating::{RatingHistory, RatingId, RatingService};
use crate::user::UserService;
use crate::wiki::{UpdateWiki, WikiService};
use chrono::prelude::*;
use diesel::{Connection, PgConnection};
use either::*;
use std::fmt::{self, Debug};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use wikidot_normalize::normalize;

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
        let user = UserService::new(&conn);
        let wiki = WikiService::new(&conn)?;

        Ok(Server {
            author,
            conn,
            page,
            password,
            rating,
            user,
            wiki,
        })
    }

    #[cfg(test)]
    #[inline]
    pub fn test_transaction<F: FnOnce() -> Result<()>>(&self, f: F) {
        self.conn.test_transaction::<_, Error, _>(f);
    }

    /* Wiki methods */

    /// Creates a new wiki with the given parameters. Returns its ID.
    pub fn create_wiki<S1, S2>(&self, name: &str, slug: S1, domain: S2) -> Result<WikiId>
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        let slug = normalize_slug(slug);
        let domain = to_lowercase(domain);

        let id = self.wiki.create(name, &slug, &domain)?;

        self.wiki.get_by_id(id, |wiki| {
            let wiki = wiki.expect("Can't find wiki object after inserting");

            self.page.add_store(&wiki)?;
            Ok(id)
        })
    }

    /// Renames the given wiki.
    /// Changing a wiki's slug is not supported.
    pub fn rename_wiki(&self, id: WikiId, new_name: &str) -> Result<()> {
        let model = UpdateWiki {
            name: Some(new_name),
            domain: None,
        };

        info!("Renaming wiki ID {} to '{}'", id, new_name);

        self.wiki.edit(id, model)?;
        Ok(())
    }

    /// Changes the associated domain for the given wiki.
    pub fn set_wiki_domain(&self, id: WikiId, new_domain: &str) -> Result<()> {
        let model = UpdateWiki {
            name: None,
            domain: Some(new_domain),
        };

        info!("Changing domain for wiki ID {} to '{}'", id, new_domain);

        self.conn.transaction::<_, Error, _>(|| {
            self.wiki.edit(id, model)?;
            self.page.set_domain(id, new_domain)?;

            Ok(())
        })
    }

    /// Gets the wiki ID with the given slug.
    /// Returns an error if the wiki doesn't exist.
    pub fn get_wiki_id<S: Into<String>>(&self, slug: S) -> Result<WikiId> {
        let slug = normalize_slug(slug);

        self.wiki.get_by_slug(&slug, |wiki| match wiki {
            Some(wiki) => Ok(wiki.id()),
            None => Err(Error::WikiNotFound),
        })
    }

    /* User methods */

    /// Creates a new user with the given name and email. Returns its ID.
    #[inline]
    pub fn create_user(&self, name: &str, email: &str, password: &str) -> Result<UserId> {
        self.conn.transaction::<_, Error, _>(|| {
            let user_id = self.user.create(name, email)?;
            self.password.set(user_id, password)?;

            Ok(user_id)
        })
    }

    /// Edits data attached to a user with the given ID.
    #[inline]
    pub fn edit_user(&self, id: UserId, changes: UserMetadata) -> Result<()> {
        self.user.edit(id, changes)
    }

    /// Get the model for a user from its ID.
    #[inline]
    pub fn get_user_from_id(&self, id: UserId) -> Result<User> {
        self.user.get_from_id(id)?.ok_or(Error::UserNotFound)
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
    pub fn get_user_from_name(&self, name: &str) -> Result<Option<User>> {
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
    pub fn mark_user_inactive(&self, id: UserId) -> Result<()> {
        self.user.mark_inactive(id, true)
    }

    /// Marks the user as "active" again, effectively un-deleting them.
    #[inline]
    pub fn mark_user_active(&self, id: UserId) -> Result<()> {
        self.user.mark_inactive(id, false)
    }

    /* Authentication methods */

    /// Sets or overwrites the given user's password.
    #[inline]
    pub fn set_user_password(&self, user_id: UserId, password: &str) -> Result<()> {
        self.password.set(user_id, password)
    }

    // TODO: return token instead of doing dummy validation
    /// Validates the password for the given user.
    /// Returns `()` on success, authentication error on failure.
    #[inline]
    pub fn validate_user_password(&self, user_id: UserId, password: &str) -> Result<()> {
        self.password.check(user_id, password)
    }

    /* Page methods */

    /// Creates a new page with the given contents and metadata.
    pub fn create_page(
        &self,
        commit: PageCommit,
        content: &[u8],
        other_authors: &[UserId],
        title: &str,
        alt_title: &str,
    ) -> Result<(PageId, RevisionId)> {
        let PageCommit { user, .. } = commit;

        // Empty string means use default
        let alt_title: Option<&str> = match alt_title {
            "" => None,
            _ => Some(alt_title),
        };

        self.conn.transaction::<_, Error, _>(|| {
            // Create page
            let (page_id, revision_id) = self.page.create(commit, content, title, alt_title)?;

            // Add committing user as author
            self.author
                .add(page_id, user.id(), AuthorType::Author, None)?;

            // Add other users
            for user_id in other_authors.iter().copied() {
                self.author
                    .add(page_id, user_id, AuthorType::Author, None)?;
            }

            Ok((page_id, revision_id))
        })
    }

    /// Edits an existing page to have the given content.
    /// Optionally permits modifying the title or alternate title.
    /// (An empty alternate title signifies that none is used)
    pub fn edit_page(
        &self,
        commit: PageCommit,
        content: Option<&[u8]>,
        title: Option<&str>,
        alt_title: Option<&str>,
    ) -> Result<RevisionId> {
        // Empty string means use default
        let alt_title: Option<Option<&str>> = match alt_title {
            Some("") => Some(None),
            Some(_) => Some(alt_title),
            None => None,
        };

        self.page.commit(commit, content, title, alt_title)
    }

    /// Renames a page to use a different slug.
    #[inline]
    pub fn rename_page<S1, S2>(
        &self,
        wiki_id: WikiId,
        old_slug: S1,
        new_slug: S2,
        message: &str,
        user: &User,
    ) -> Result<RevisionId>
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        let old_slug = normalize_slug(old_slug);
        let new_slug = normalize_slug(new_slug);

        self.page
            .rename(wiki_id, &old_slug, &new_slug, message, user)
    }

    /// Removes the given page.
    #[inline]
    pub fn remove_page(&self, commit: PageCommit) -> Result<RevisionId> {
        self.page.remove(commit)
    }

    /// Determines if a page with the given slug exists.
    #[inline]
    pub fn check_page<S: Into<String>>(&self, wiki_id: WikiId, slug: S) -> Result<bool> {
        let slug = normalize_slug(slug);

        self.page.check_page(wiki_id, &slug)
    }

    /// Gets the metadata for a given page, as well as its rating information.
    /// Uses Wikidot's `ups - downs` formula for scoring.
    pub fn get_page<S: Into<String>>(
        &self,
        wiki_id: WikiId,
        slug: S,
    ) -> Result<Option<(Page, Rating)>> {
        debug!("Creating transaction for page and rating");

        let slug = normalize_slug(slug);

        self.conn.transaction::<_, Error, _>(|| {
            let page = match self.page.get_page(wiki_id, &slug)? {
                Some(page) => page,
                None => return Ok(None),
            };

            let rating = self.rating.get_rating(page.id())?;

            Ok(Some((page, rating)))
        })
    }

    /// Gets the metadata for a given page ID, as well as its rating information.
    /// Uses Wikidot's `ups - downs` formula for scoring.
    pub fn get_page_by_id(&self, page_id: PageId) -> Result<Option<(Page, Rating)>> {
        debug!("Creating transaction for page ID and rating");

        self.conn.transaction::<_, Error, _>(|| {
            let page = match self.page.get_page_by_id(page_id)? {
                Some(page) => page,
                None => return Ok(None),
            };

            let rating = self.rating.get_rating(page_id)?;

            Ok(Some((page, rating)))
        })
    }

    /// Gets the contents for a given page.
    #[inline]
    pub fn get_page_contents<S: Into<String>>(
        &self,
        wiki_id: WikiId,
        slug: S,
    ) -> Result<Option<Box<[u8]>>> {
        let slug = normalize_slug(slug);

        self.page.get_page_contents(wiki_id, &slug)
    }

    /// Gets the contents for a given page ID.
    #[inline]
    pub fn get_page_contents_by_id(&self, page_id: PageId) -> Result<Option<Box<[u8]>>> {
        self.page.get_page_contents_by_id(page_id)
    }

    /// Sets all the tags for a given page.
    #[inline]
    pub fn set_page_tags<S: AsRef<str>>(
        &self,
        commit: PageCommit,
        tags: &[S],
    ) -> Result<RevisionId> {
        let mut tags = tags.iter().map(|tag| tag.as_ref()).collect::<Vec<&str>>();

        self.page.tags(commit, &mut tags)
    }

    /* Author methods */

    fn get_page_id<S: Into<String>>(&self, page: Either<PageId, (WikiId, S)>) -> Result<PageId> {
        match page {
            Left(id) => Ok(id),
            Right((wiki_id, slug)) => {
                let slug = normalize_slug(slug);

                self.page
                    .get_page_id(wiki_id, &slug)?
                    .ok_or(Error::PageNotFound)
            }
        }
    }

    /// Gets all authors for a given page.
    pub fn get_page_authors(&self, page: Either<PageId, (WikiId, &str)>) -> Result<Vec<Author>> {
        info!("Getting authors for page {:?}", page);

        self.conn.transaction::<_, Error, _>(|| {
            let page_id = self.get_page_id(page)?;

            self.author.get_all(page_id)
        })
    }

    /// Adds or sets a group of authors.
    pub fn add_page_authors(
        &self,
        page: Either<PageId, (WikiId, &str)>,
        authors: &[(UserId, AuthorType, Option<NaiveDate>)],
    ) -> Result<()> {
        info!("Adding authors to page {:?}: {:?}", page, authors);

        self.conn.transaction::<_, Error, _>(|| {
            let page_id = self.get_page_id(page)?;

            for &(user_id, author_type, written_at) in authors {
                self.author.add(page_id, user_id, author_type, written_at)?;
            }

            Ok(())
        })
    }

    /// Removes a group of authors.
    pub fn remove_page_authors(
        &self,
        page: Either<PageId, (WikiId, &str)>,
        authors: &[(UserId, AuthorType)],
    ) -> Result<usize> {
        info!("Removing authors from page {:?}: {:?}", page, authors);

        self.conn.transaction::<_, Error, _>(|| {
            let page_id = self.get_page_id(page)?;
            let mut count = 0;

            for (user_id, author_type) in authors.iter().copied() {
                if self.author.remove(page_id, user_id, author_type)? {
                    count += 1;
                }
            }

            Ok(count)
        })
    }

    /* Rating methods */

    /// Sets the rating for a given page and user.
    #[inline]
    pub fn set_rating(&self, page_id: PageId, user_id: UserId, rating: i16) -> Result<RatingId> {
        info!(
            "Setting rating for page ID {} / user ID {}: {}",
            page_id, user_id, rating,
        );

        self.rating.set(page_id, user_id, rating)
    }

    /// Removes the rating for a given page and user.
    /// Returns `None` if the rating is already deleted.
    #[inline]
    pub fn remove_rating(&self, page_id: PageId, user_id: UserId) -> Result<Option<RatingId>> {
        info!(
            "Removing rating for page ID {} / user ID {}",
            page_id, user_id,
        );

        self.rating.remove(page_id, user_id)
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
    pub fn get_page_diff<S: Into<String>>(
        &self,
        wiki_id: WikiId,
        slug: S,
        first: Either<RevisionId, GitHash>,
        second: Either<RevisionId, GitHash>,
    ) -> Result<Box<[u8]>> {
        let slug = normalize_slug(slug);

        self.page.get_diff(wiki_id, &slug, first, second)
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

fn normalize_slug<S: Into<String>>(slug: S) -> String {
    let mut slug = slug.into();
    normalize(&mut slug);
    slug
}

fn to_lowercase<S: Into<String>>(value: S) -> String {
    let mut value = value.into();
    value.make_ascii_lowercase();
    value
}
