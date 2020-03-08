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
pub struct AverageScorer;

impl Scoring for AverageScorer {
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
    macro_rules! check {
        ($votes:expr, $score:expr) => {
            assert_eq!(AverageScorer::score(&*$votes), $score, "Score mismatch");
        };
    }

    check!(NO_VOTES, 0);
    check!(POSITIVE_VOTES, 0);
    check!(POSITIVE_AND_NEUTRAL_VOTES, 0);
    check!(NEGATIVE_VOTES, 0);
    check!(NEUTRAL_VOTES, 0);
    check!(MIXED_VOTES_1, 0);
    check!(MIXED_VOTES_2, 0);
}
