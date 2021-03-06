/*
 * scoring/null.rs
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

/// Null-scorer. Returns `0` for all pages.
#[derive(Debug, Copy, Clone, Default)]
pub struct NullScoring;

impl Scoring for NullScoring {
    fn score(_votes: &Votes) -> f32 {
        0.0
    }
}

#[test]
fn null_scoring() {
    macro_rules! check {
        ($votes:expr) => {
            f32_eq(NullScoring::score(&*$votes), 0.0, 0.0000001);
        };
    }

    check!(NO_VOTES);
    check!(POSITIVE_VOTES);
    check!(POSITIVE_AND_NEUTRAL_VOTES);
    check!(NEGATIVE_VOTES);
    check!(NEUTRAL_VOTES);
    check!(MIXED_VOTES_1);
    check!(MIXED_VOTES_2);
}
