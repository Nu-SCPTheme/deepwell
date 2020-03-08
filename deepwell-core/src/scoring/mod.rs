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

mod average;
mod null;
mod percent;
mod wikidot;
mod wilson;

mod prelude {
    pub use super::Scoring;
    pub use crate::models::Votes;

    cfg_if! {
        if #[cfg(test)] {
            pub use super::f32_eq;

            use map_vec::Map;

            lazy_static! {
                pub static ref NO_VOTES: Votes = Votes::new(Map::new());
                pub static ref POSITIVE_VOTES: Votes = {
                    let mut distr = Map::new();
                    distr.insert(1, 20);
                    Votes::new(distr)
                };
                pub static ref POSITIVE_AND_NEUTRAL_VOTES: Votes = {
                    let mut distr = Map::new();
                    distr.insert(1, 12);
                    distr.insert(0, 8);
                    Votes::new(distr)
                };
                pub static ref NEGATIVE_VOTES: Votes = {
                    let mut distr = Map::new();
                    distr.insert(-1, 5);
                    Votes::new(distr)
                };
                pub static ref NEUTRAL_VOTES: Votes = {
                    let mut distr = Map::new();
                    distr.insert(0, 8);
                    Votes::new(distr)
                };
                pub static ref MIXED_VOTES_1: Votes = {
                    let mut distr = Map::new();
                    distr.insert(1, 46);
                    distr.insert(0, 18);
                    distr.insert(-1, 20);
                    Votes::new(distr)
                };
                pub static ref MIXED_VOTES_2: Votes = {
                    let mut distr = Map::new();
                    distr.insert(1, 20);
                    distr.insert(0, 36);
                    distr.insert(-1, 15);
                    Votes::new(distr)
                };
            }
        }
    }
}

pub use self::average::AverageScorer;
pub use self::null::NullScoring;
pub use self::percent::PercentScoring;
pub use self::wikidot::WikidotScoring;
pub use self::wilson::WilsonScoring;

/// Trait for determining the rating from votes.
///
/// Allows for different implementations, at the choice of wiki
/// administrators.
pub trait Scoring {
    fn score(votes: &Votes) -> f32;
}

#[cfg(test)]
pub fn f32_eq(x: f32, y: f32) {
    assert!((x - y).abs() < 0.000001, "Score mismatch")
}
