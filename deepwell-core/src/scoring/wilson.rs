/*
 * scoring/wilson.rs
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

use super::prelude::*;

/// Lower bound of a Wilson score confidence interval.
/// See https://www.evanmiller.org/how-not-to-sort-by-average-rating.html
#[derive(Debug, Copy, Clone, Default)]
pub struct WilsonScoring;

impl Scoring for WilsonScoring {
    fn score(votes: &Votes) -> i32 {
        macro_rules! get_votes {
            (+1) => {
                get_votes!(1)
            };

            ($value:expr) => {
                *votes.distribution().get(&$value).unwrap_or(&0) as f32
            };
        }

        let total = votes.count() as f32;
        let positive = get_votes!(+1);
        let negative = get_votes!(-1);

        let a = (positive + 1.9208) / total - 1.96;
        let b = (positive * negative) / total + 0.9604;
        let bound = (a * b.sqrt() / total) / (3.8416 / total + 1.0);

        bound as i32
    }
}
