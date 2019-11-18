/*
 * server/revision.rs
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

use super::utils::normalize_slug;
use crate::service_prelude::*;

impl Server {
    /// Get the version of a page at the specified revision.
    #[inline]
    pub async fn get_page_version(
        &self,
        wiki_id: WikiId,
        slug: &str,
        revision: Either<RevisionId, &GitHash>,
    ) -> Result<Option<Box<[u8]>>> {
        self.page.get_page_version(wiki_id, slug, revision).await
    }

    /// Get the blame for a given page, if it exists.
    #[inline]
    pub async fn get_page_blame(&self, wiki_id: WikiId, slug: &str) -> Result<Option<Blame>> {
        self.page.get_blame(wiki_id, slug).await
    }

    /// Get the blame for a given page ID.
    #[inline]
    pub async fn get_page_blame_by_id(&self, page_id: PageId) -> Result<Option<Blame>> {
        self.page.get_blame_by_id(page_id).await
    }

    /// Get a diff for a given page between the two specified revisions.
    #[inline]
    pub async fn get_page_diff<S: Into<String>>(
        &self,
        wiki_id: WikiId,
        slug: S,
        first: Either<RevisionId, &GitHash>,
        second: Either<RevisionId, &GitHash>,
    ) -> Result<Box<[u8]>> {
        let slug = normalize_slug(slug);

        self.page.get_diff(wiki_id, &slug, first, second).await
    }

    /// Overwrite the revision message for a given change.
    #[inline]
    pub async fn edit_revision(&self, revision_id: RevisionId, message: &str) -> Result<()> {
        self.page.edit_revision(revision_id, message).await
    }
}
