/*
 * server/page.rs
 *
 * deepwell - Database management and migrations service
 * Copyright (C) 2019-2020 Ammon Smith
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

use super::utils::normalize_slug;
use crate::author::AuthorType;
use crate::manager_prelude::*;

impl Server {
    /// Creates a new page with the given contents and metadata.
    pub async fn create_page(
        &self,
        commit: PageCommit<'_>,
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

        self.transaction(async {
            // Create page
            let (page_id, revision_id) =
                self.page.create(commit, content, title, alt_title).await?;

            // Add committing user as author
            self.author
                .add(page_id, user.id(), AuthorType::Author, None)
                .await?;

            // Add other users
            for user_id in other_authors.iter().copied() {
                self.author
                    .add(page_id, user_id, AuthorType::Author, None)
                    .await?;
            }

            Ok((page_id, revision_id))
        })
        .await
    }

    /// Edits an existing page to have the given content.
    /// Optionally permits modifying the title or alternate title.
    /// (An empty alternate title signifies that none is used)
    pub async fn edit_page(
        &self,
        commit: PageCommit<'_>,
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

        self.page.commit(commit, content, title, alt_title).await
    }

    /// Renames a page to use a different slug.
    #[inline]
    pub async fn rename_page<S1, S2>(
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
            .await
    }

    /// Removes the given page.
    #[inline]
    pub async fn remove_page(&self, commit: PageCommit<'_>) -> Result<RevisionId> {
        self.page.remove(commit).await
    }

    /// Determines if a page with the given slug exists.
    #[inline]
    pub async fn check_page<S: Into<String>>(&self, wiki_id: WikiId, slug: S) -> Result<bool> {
        let slug = normalize_slug(slug);

        self.page.check_page(wiki_id, &slug).await
    }

    /// Gets the metadata for a given page, as well as its rating information.
    /// Uses Wikidot's `ups - downs` formula for scoring.
    pub async fn get_page<S: Into<String>>(
        &self,
        wiki_id: WikiId,
        slug: S,
    ) -> Result<Option<(Page, Rating)>> {
        debug!("Creating transaction for page and rating");

        let slug = normalize_slug(slug);

        self.transaction(async {
            let result = self.page.get_page(wiki_id, &slug).await?;
            let page = match result {
                Some(page) => page,
                None => return Ok(None),
            };

            let page_id = page.id();
            let rating = self.rating.get_rating(page_id).await?;

            Ok(Some((page, rating)))
        })
        .await
    }

    /// Gets the metadata for a given page ID, as well as its rating information.
    /// Uses Wikidot's `ups - downs` formula for scoring.
    pub async fn get_page_by_id(&self, page_id: PageId) -> Result<Option<(Page, Rating)>> {
        debug!("Creating transaction for page ID and rating");

        self.transaction(async {
            let result = self.page.get_page_by_id(page_id).await?;
            let page = match result {
                Some(page) => page,
                None => return Ok(None),
            };

            let rating = self.rating.get_rating(page_id).await?;

            Ok(Some((page, rating)))
        })
        .await
    }

    /// Gets the contents for a given page.
    #[inline]
    pub async fn get_page_contents<S: Into<String>>(
        &self,
        wiki_id: WikiId,
        slug: S,
    ) -> Result<Option<Box<[u8]>>> {
        let slug = normalize_slug(slug);

        self.page.get_page_contents(wiki_id, &slug).await
    }

    /// Gets the contents for a given page ID.
    #[inline]
    pub async fn get_page_contents_by_id(&self, page_id: PageId) -> Result<Option<Box<[u8]>>> {
        self.page.get_page_contents_by_id(page_id).await
    }

    /// Sets all the tags for a given page.
    #[inline]
    pub async fn set_page_tags<S: AsRef<str>>(
        &self,
        commit: PageCommit<'_>,
        tags: &[S],
    ) -> Result<RevisionId> {
        // Allow tags to be sorted before insertion
        let mut tags = tags.iter().map(|tag| tag.as_ref()).collect::<Vec<&str>>();

        self.page.tags(commit, &mut tags).await
    }

    /// Gets all pages which have at least the given tags.
    ///
    /// Returns an empty set if no tags are passed in.
    #[inline]
    pub async fn get_pages_with_tags(&self, wiki_id: WikiId, tags: &[&str]) -> Result<Vec<Page>> {
        self.page.get_pages_with_tags(wiki_id, tags).await
    }
}
