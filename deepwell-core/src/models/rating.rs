/*
 * models/rating.rs
 *
 * deepwell-core - Database management and migrations service
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

use map_vec::Map;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Rating {
    /// Number of each kind of vote.
    ///
    /// For instance a page with two +1 and three -1 votes would be:
    /// ```ignore
    /// {
    ///     1: 2,
    ///     -1: 3,
    /// }
    /// ```
    votes: Map<i16, u32>,

    /// Total number of votes.
    count: u32,
}

impl Rating {
    #[inline]
    pub fn new(votes: Map<i16, u32>) -> Self {
        let count = votes.values().sum();

        Rating { votes, count }
    }

    #[inline]
    pub fn votes(&self) -> &Map<i16, u32> {
        &self.votes
    }

    #[inline]
    pub fn count(&self) -> u32 {
        self.count
    }
}
