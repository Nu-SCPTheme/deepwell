/*
 * scoring/wikidot.rs
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

/// Wikidot-compatible scorer. Returns the sum of all votes.
/// Equivalent to `ups - downs`.
#[derive(Debug, Copy, Clone, Default)]
pub struct WikidotScoring;

impl Scoring for WikidotScoring {
    fn score(votes: &Votes) -> f32 {
        votes.iter().fold(0.0, |score, (vote, count)| {
            let vote = f32::from(vote);
            let count = count as f32;

            score + vote * count
        })
    }
}

#[test]
fn wikidot_scoring() {
    macro_rules! check {
        ($votes:expr, $score:expr) => {
            f32_eq(WikidotScoring::score(&*$votes), $score, 0.1);
        };
    }

    check!(NO_VOTES, 0.0);
    check!(POSITIVE_VOTES, 20.0);
    check!(POSITIVE_AND_NEUTRAL_VOTES, 12.0);
    check!(NEGATIVE_VOTES, -5.0);
    check!(NEUTRAL_VOTES, 0.0);
    check!(MIXED_VOTES_1, 26.0);
    check!(MIXED_VOTES_2, 5.0);
}
