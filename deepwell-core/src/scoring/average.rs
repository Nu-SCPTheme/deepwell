/*
 * scoring/average.rs
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
use super::WikidotScoring;

/// Average-based scorer. Returns the mean of all the votes cast.
#[derive(Debug, Copy, Clone, Default)]
pub struct AverageScoring;

impl Scoring for AverageScoring {
    fn score(votes: &Votes) -> f32 {
        if votes.count() == 0 {
            return 0.0;
        }

        let sum = WikidotScoring::score(votes);
        let total = votes.count() as f32;

        sum / total
    }
}

#[test]
fn average_voting() {
    use map_vec::Map;

    lazy_static! {
        static ref FIVE_STAR_1: Votes = {
            let mut distr = Map::new();
            distr.insert(1, 48);
            distr.insert(2, 64);
            distr.insert(3, 50);
            distr.insert(4, 21);
            distr.insert(5, 33);
            Votes::new(distr)
        };
        static ref FIVE_STAR_2: Votes = {
            let mut distr = Map::new();
            distr.insert(1, 12);
            distr.insert(2, 3);
            distr.insert(3, 3);
            distr.insert(4, 28);
            distr.insert(5, 41);
            Votes::new(distr)
        };
        static ref FIVE_STAR_3: Votes = {
            let mut distr = Map::new();
            distr.insert(1, 0);
            distr.insert(2, 0);
            distr.insert(3, 0);
            distr.insert(4, 0);
            distr.insert(5, 10);
            Votes::new(distr)
        };
    }

    macro_rules! check {
        ($votes:expr, $score:expr) => {
            f32_eq(AverageScoring::score(&*$votes), $score, 0.1);
        };
    }

    check!(NO_VOTES, 0.0);
    check!(POSITIVE_VOTES, 1.0);
    check!(POSITIVE_AND_NEUTRAL_VOTES, 0.6);
    check!(NEGATIVE_VOTES, -1.0);
    check!(NEUTRAL_VOTES, 0.0);
    check!(MIXED_VOTES_1, 0.3);
    check!(MIXED_VOTES_2, 0.0);

    check!(FIVE_STAR_1, 2.6);
    check!(FIVE_STAR_2, 3.9);
    check!(FIVE_STAR_3, 5.0);
}
