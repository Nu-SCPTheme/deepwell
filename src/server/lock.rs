/*
 * server/lock.rs
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

use crate::manager_prelude::*;

impl Server {
    /// Removes any page locks which are no longer active.
    pub async fn invalidate_expired_locks(&self) -> Result<usize> {
        self.lock.invalidate_expired().await
    }

    /// Creates a page lock for the given user.
    ///
    /// The amount of time to acquire the lock for is dependent on the wiki's settings.
    /// This will fail if a lock is already held for this page.
    pub async fn create_page_lock(
        &self,
        wiki_id: WikiId,
        page_id: PageId,
        user_id: UserId,
    ) -> Result<()> {
        self.transaction(async {
            unimplemented!()
        })
        .await
    }

    /// Extends the page lock for the given user.
    ///
    /// This will fail if this user does not currently hold a lock for this page.
    pub async fn update_page_lock(
        &self,
        wiki_id: WikiId,
        page_id: PageId,
        user_id: UserId,
    ) -> Result<()> {
        unimplemented!()
    }

    /// Lifts the page lock for a particular page.
    ///
    /// This will fail if there is no page lock present.
    #[inline]
    pub async fn remove_page_lock(&self, page_id: PageId) -> Result<()> {
        self.lock.remove(page_id).await
    }
}
