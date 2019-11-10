/*
 * server/rating.rs
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

use crate::rating::{RatingHistory, RatingId};
use crate::service_prelude::*;

impl Server {
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
}
