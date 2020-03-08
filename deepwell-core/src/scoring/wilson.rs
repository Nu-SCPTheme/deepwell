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
use crate::math::probit;

/// Lower bound of a Wilson score confidence interval.
/// See https://www.evanmiller.org/how-not-to-sort-by-average-rating.html and
/// https://www.itl.nist.gov/div898/handbook/prc/section2/prc241.htm
///
/// Takes implementation from https://github.com/simple-statistics/simple-statistics
#[derive(Debug, Copy, Clone, Default)]
pub struct WilsonScoring;

impl Scoring for WilsonScoring {
    fn score(votes: &Votes) -> f32 {
        const CONFIDENCE: f32 = 0.95;

        // Note: while implementation matches
        // https://medium.com/@gattermeier/calculating-better-rating-scores-for-things-voted-on-7fa3f632c79d
        // it could definitely use some tuning, especially with regards to neutral-vote and overall
        // score, which is presently multipled my total votes.

        if votes.count() == 0 {
            return 0.0;
        }

        let positive = votes.count_for_vote(1).unwrap_or(0) as f32;
        let total = votes.count() as f32;

        let p_hat = 1.0 * positive / total;
        let z = probit(1.0 - (1.0 - CONFIDENCE) / 2.0);
        let z_2 = z * z;

        let a = p_hat + z_2 / (2.0 * total);
        let b = p_hat * (1.0 - p_hat) + z_2 / (4.0 * total);
        let lower_bound = (a - z * (b / total).sqrt()) / (1.0 + z_2 / total);

        lower_bound * total
    }
}

#[test]
fn wilson_scoring() {
    macro_rules! check {
        ($votes:expr, $score:expr) => {
            assert_eq!(WilsonScoring::score(&*$votes), $score, "Score mismatch");
        };
    }

    check!(NO_VOTES, 0);
    check!(POSITIVE_VOTES, 16);
    check!(POSITIVE_AND_NEUTRAL_VOTES, 7);
    check!(NEGATIVE_VOTES, 0);
    check!(NEUTRAL_VOTES, 0);
    check!(MIXED_VOTES_1, 37);
    check!(MIXED_VOTES_2, 13);
}
