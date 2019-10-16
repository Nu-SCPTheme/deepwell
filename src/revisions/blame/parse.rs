/*
 * revisions/blame/parse.rs
 *
 * deepwell - Database management and migrations service
 * Copyright (C) 2019 Ammon Smith
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

use super::*;
use crate::{Error, Result};
use crate::revisions::GitHash;
use chrono::{DateTime, FixedOffset};
use regex::bytes::Regex;
use std::str;

lazy_static! {
    static ref GIT_HASH_REGEX: Regex = Regex::new(r"
        (?x)
        ^
        (?P<sha1>[0-9a-f]{40})
        \s
        (?P<old-line>[0-9]+)
        \s
        (?P<new-line>[0-9]+)
        (\s(?P<group-lines>[0-9]+))?
        $
    ").unwrap();

    static ref METADATA_REGEX: Regex = Regex::new(r"
        (?x)
        ^
        (?P<key>[a-z\-]+
        (\s(?P<value>.+))?
        $
    ").unwrap();

    static ref CONTENT_REGEX: Regex = Regex::new("\t(?P<content>.*)").unwrap();
}

#[derive(Debug, Copy, Clone)]
enum State {
    /// The first line in a group, where the git hash and range are.
    Commit,

    /// Gathering various pieces of metadata about commit information.
    Headers,

    /// The line of actual file data, as it exists.
    Content,
}

#[derive(Debug, Default)]
struct Author {
    name: String,
    email: String,
    time: u64,
    tz: i32,
}

impl Blame {
    pub fn from_porcelain(raw_bytes: &[u8]) -> Result<Self> {
        const BLAME_ERROR: Error = Error::StaticMsg("unexpected or mismatched input line in blame data");

        macro_rules! capture {
            ($regex:expr, $line:expr) => {
                match $regex.captures($line) {
                    Some(captures) => captures,
                    None => return Err(BLAME_ERROR),
                }
            }
        }

        macro_rules! utf {
            ($captures:expr, $name:expr) => (
                str::from_utf8(&$captures[$name]).unwrap()
            )
        }

        let lines = raw_bytes.split(|&b| b == b'\n');
        let mut state = State::Headers;

        let mut author = Author::default();
        let mut committer = Author::default();
        let mut summary = String::new();

        let mut blame_lines = Vec::new();

        for line in lines {
            match state {
                State::Commit => {
                    let captures = capture!(GIT_HASH_REGEX, line);

                    let old_lineno = utf!(captures, "old-line").parse().unwrap();
                    let new_lineno = utf!(captures, "new-line").parse().unwrap();
                    let commit = {
                        let sha1 = utf!(captures, "sha1");
                        GitHash::from_str(sha1).unwrap()
                    };

                    blame_lines.push(BlameLine {
                        commit,
                        old_lineno,
                        new_lineno,
                    });

                    state = State::Headers;
                }
            }
        }

        unimplemented!()
    }
}
