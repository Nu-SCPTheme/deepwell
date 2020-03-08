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
    fn score(votes: &Votes) -> i32 {
        macro_rules! get_vote {
            ($vote:expr) => {
                votes.count_for_vote($vote).unwrap_or(0) as f32
            };
        }

        if votes.count() == 0 {
            return 0;
        }

        let positive = get_vote!(1);
        let neutral = get_vote!(0) * 0.5;
        let total = votes.count() as f32;

        let score = (positive + neutral) / total;
        (score * 100.0) as i32
    }
}

#[test]
fn percent_voting() {
    macro_rules! check {
        ($votes:expr, $score:expr) => {
            assert_eq!(PercentScoring::score(&*$votes), $score, "Score mismatch");
        };
    }

    check!(NO_VOTES, 0);
    check!(POSITIVE_VOTES, 100);
    check!(POSITIVE_AND_NEUTRAL_VOTES, 80);
    check!(NEGATIVE_VOTES, 0);
    check!(NEUTRAL_VOTES, 50);
    check!(MIXED_VOTES_1, 65);
    check!(MIXED_VOTES_2, 53);
}
