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

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Rating {
    score: i64,
    votes: u32,
}

impl Rating {
    #[inline]
    pub fn new(score: i64, votes: u32) -> Self {
        Rating { score, votes }
    }

    #[inline]
    pub fn score(&self) -> i64 {
        self.score
    }

    #[inline]
    pub fn votes(&self) -> u32 {
        self.votes
    }
}
