/*
 * scoring/mod.rs
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

use crate::models::Votes;

mod null;
mod wikidot;
mod wilson;

mod prelude {
    pub use crate::models::Votes;
    pub use super::Scoring;
}

pub use self::null::NullScoring;
pub use self::wikidot::WikidotScoring;

/// Trait for determining the rating from votes.
///
/// Allows for different implementations, at the choice of wiki
/// administrators.
pub trait Scoring {
    fn score(votes: &Votes) -> i32;
}
