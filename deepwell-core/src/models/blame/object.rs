/*
 * models/blame/object.rs
 *
 * deepwell - Database management and migrations service
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

use crate::models::GitHash;
use chrono::{DateTime, FixedOffset};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlameAuthor {
    pub name: String,
    pub email: String,
    pub time: DateTime<FixedOffset>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlameLine {
    pub commit: GitHash,
    pub old_lineno: u32,
    pub new_lineno: u32,
    pub line: Box<[u8]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlameGroup {
    pub author: BlameAuthor,
    pub committer: BlameAuthor,
    pub summary: String,
    pub previous: Option<GitHash>,
    pub lines: Vec<BlameLine>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Blame {
    pub groups: Vec<BlameGroup>,
}
