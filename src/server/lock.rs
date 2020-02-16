/*
 * server/lock.rs
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
use crate::manager_prelude::*;

impl Server {
    /// Removes any page locks which are no longer active.
    pub async fn invalidate_expired_locks(&self) -> Result<usize> {
        self.lock.invalidate_expired().await
    }

    async fn lock_page_id(&self, wiki_id: WikiId, slug: &str) -> Result<PageId> {
        self.page
            .get_page_id(wiki_id, &slug)
            .await?
            .ok_or(Error::PageNotFound)
    }

    async fn lock_duration(&self, wiki_id: WikiId) -> Result<chrono::Duration> {
        let settings = self.wiki.get_settings(wiki_id).await?;
        let duration = settings.page_lock_duration();

        Ok(duration)
    }

    /// Creates a page lock for the given user.
    ///
    /// The amount of time to acquire the lock for is dependent on the wiki's settings.
    /// This will fail if a lock is already held for this page.
    pub async fn create_page_lock<S: Into<String>>(
        &self,
        wiki_id: WikiId,
        slug: S,
        user_id: UserId,
    ) -> Result<()> {
        let slug = normalize_slug(slug);

        info!(
            "Creating page lock for wiki ID {} / slug '{}' for user ID {}",
            wiki_id, slug, user_id,
        );

        self.transaction(async {
            let page_id = self.lock_page_id(wiki_id, &slug).await?;

            self.lock.check(page_id, user_id).await?;

            let lock_duration = self.lock_duration(wiki_id).await?;
            self.lock.add(page_id, user_id, lock_duration).await?;

            Ok(())
        })
        .await
    }

    /// Extends the page lock for the given user.
    ///
    /// This will fail if this user does not currently hold a lock for this page.
    pub async fn update_page_lock(
        &self,
        wiki_id: WikiId,
        slug: &str,
        user_id: UserId,
    ) -> Result<()> {
        let slug = normalize_slug(slug);

        info!(
            "Updating page lock for wiki ID {} / slug '{}' for user ID {}",
            wiki_id, slug, user_id,
        );

        self.transaction(async {
            let page_id = self.lock_page_id(wiki_id, &slug).await?;

            self.lock.check(page_id, user_id).await?;

            let lock_duration = self.lock_duration(wiki_id).await?;
            self.lock.update(page_id, user_id, lock_duration).await?;

            Ok(())
        })
        .await
    }

    /// Lifts the page lock for a particular page.
    ///
    /// This will fail if there is no page lock present.
    pub async fn remove_page_lock<S: Into<String>>(&self, wiki_id: WikiId, slug: S) -> Result<()> {
        let slug = normalize_slug(slug);

        info!(
            "Removing page lock for wiki ID {} / slug '{}'",
            wiki_id, slug,
        );

        let page_id = self.lock_page_id(wiki_id, &slug).await?;

        self.lock.remove(page_id).await
    }
}
