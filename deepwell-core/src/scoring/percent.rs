/*
 * scoring/percent.rs
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

/// Percent-based scorer. Gives the percentage of votes which were upvotes,
/// treating neutral-votes as half an upvote.
#[derive(Debug, Copy, Clone, Default)]
pub struct PercentScoring;

impl Scoring for PercentScoring {
    fn score(votes: &Votes) -> f32 {
        macro_rules! get_vote {
            ($vote:expr) => {
                votes.count_for_vote($vote).unwrap_or(0) as f32
            };
        }

        if votes.count() == 0 {
            return 0.0;
        }

        let positive = get_vote!(1);
        let neutral = get_vote!(0) * 0.5;
        let total = votes.count() as f32;

        (positive + neutral) / total * 100.0
    }
}

#[test]
fn percent_voting() {
    macro_rules! check {
        ($votes:expr, $score:expr) => {
            f32_eq(PercentScoring::score(&*$votes), $score, 0.1);
        };
    }

    check!(NO_VOTES, 0.0);
    check!(POSITIVE_VOTES, 100.0);
    check!(POSITIVE_AND_NEUTRAL_VOTES, 80.0);
    check!(NEGATIVE_VOTES, 0.0);
    check!(NEUTRAL_VOTES, 50.0);
    check!(MIXED_VOTES_1, 65.5);
    check!(MIXED_VOTES_2, 53.5);
}
