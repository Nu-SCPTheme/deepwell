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
use crate::revisions::GitHash;
use crate::{Error, Result};
use chrono::{DateTime, FixedOffset, NaiveDateTime};
use regex::bytes::Regex;
use std::{mem, str};

// TODO split into separate crate

lazy_static! {
    static ref GIT_HASH_REGEX: Regex = Regex::new(
        r"(?x)
        ^
        (?P<sha1>[0-9a-f]{40})
        \s
        (?P<old_line>[0-9]+)
        \s
        (?P<new_line>[0-9]+)
        (\s(?P<group_lines>[0-9]+))?
        $
    "
    )
    .unwrap();
    static ref METADATA_REGEX: Regex = Regex::new(
        r"(?x)
        ^
        (?P<key>[a-z\-]+)
        (\s(?P<value>.+))?
        $
    "
    )
    .unwrap();
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
    timestamp: i64,
    tz: i32,
}

impl Into<BlameAuthor> for Author {
    fn into(self) -> BlameAuthor {
        let Author {
            name,
            email,
            timestamp,
            tz,
        } = self;

        let tz_secs = (tz * 60) / 100;
        let offset = FixedOffset::west(tz_secs);
        let time_naive = NaiveDateTime::from_timestamp(timestamp, 0);
        let time = DateTime::from_utc(time_naive, offset);

        BlameAuthor { name, email, time }
    }
}

// Blame implementation

impl Blame {
    pub fn from_porcelain(raw_bytes: &[u8]) -> Result<Self> {
        const BLAME_ERROR: Error =
            Error::StaticMsg("unexpected or mismatched input line in blame data");

        macro_rules! utf {
            ($captures:expr, $name:expr) => {
                str::from_utf8(&$captures[$name]).unwrap()
            };
        }

        macro_rules! set_string {
            ($field:expr, $value:expr) => {{
                $field.clear();
                $field.push_str($value);
            }};
        }

        // FSM state
        let lines = raw_bytes.split(|&b| b == b'\n');
        let mut state = State::Commit;
        let mut new_group = false;

        // Temporary state to build next item
        let mut author = Author::default();
        let mut committer = Author::default();
        let mut summary = String::new();
        let mut previous_commit = None;
        let mut commit_info = None;

        // In-progress result data
        let mut blame_groups = Vec::new();
        let mut blame_lines = Vec::new();

        for line in lines {
            if line.is_empty() {
                continue;
            }

            if line.starts_with(b"\t") {
                state = State::Content;
            }

            match state {
                State::Commit => {
                    let captures = match GIT_HASH_REGEX.captures(line) {
                        Some(captures) => captures,
                        None => return Err(BLAME_ERROR),
                    };

                    // Unwraps are safe because the values are regex-verified
                    let old_lineno = utf!(captures, "old_line").parse().unwrap();
                    let new_lineno = utf!(captures, "new_line").parse().unwrap();
                    let commit = {
                        let sha1 = utf!(captures, "sha1");
                        GitHash::parse_str(sha1).unwrap()
                    };

                    commit_info = Some((commit, old_lineno, new_lineno));
                    state = State::Headers;
                }
                State::Headers => {
                    let captures = match METADATA_REGEX.captures(line) {
                        Some(captures) => captures,
                        None => {
                            new_group = true;
                            state = State::Content;
                            continue;
                        }
                    };

                    let key = utf!(captures, "key");
                    let value = captures
                        .name("value")
                        .map(|mtch| str::from_utf8(mtch.as_bytes()).unwrap());

                    match key {
                        "author" => {
                            let value = value.expect("No value for key author");
                            set_string!(&mut author.name, value);
                        }
                        "author-mail" => {
                            let value = value.expect("No value for key author-mail");
                            set_string!(&mut author.email, value);
                        }
                        "author-time" => {
                            let value = value.expect("No value for key author-time");
                            author.timestamp = value.parse().unwrap();
                        }
                        "author-tz" => {
                            let value = value.expect("No value for key author-tz");
                            author.tz = value.parse().unwrap();
                        }
                        "committer" => {
                            let value = value.expect("No value for key author");
                            set_string!(&mut committer.name, value);
                        }
                        "committer-mail" => {
                            let value = value.expect("No value for key author-mail");
                            set_string!(&mut committer.email, value);
                        }
                        "committer-time" => {
                            let value = value.expect("No value for key author-time");
                            committer.timestamp = value.parse().unwrap();
                        }
                        "committer-tz" => {
                            let value = value.expect("No value for key author-tz");
                            committer.tz = value.parse().unwrap();
                        }
                        "summary" => {
                            let value = value.expect("No value for key summary");
                            set_string!(summary, value);
                        }
                        "previous" => {
                            let (value, _) = value.expect("No value for key previous").split_at(40);
                            let hash = GitHash::parse_str(value).expect("Unable to parse git hash");
                            previous_commit = Some(hash);
                        }
                        "boundary" => (),
                        "filename" => state = State::Content,
                        _ => (),
                    }
                }
                State::Content => {
                    let (first, line) = line.split_at(1);
                    assert_eq!(first, b"\t", "In content state but doesn't start with tab");

                    // Push new blame line
                    let line = line.into();
                    let (commit, old_lineno, new_lineno) = match commit_info.take() {
                        Some(values) => values,
                        None => return Err(BLAME_ERROR),
                    };

                    blame_lines.push(BlameLine {
                        commit,
                        old_lineno,
                        new_lineno,
                        line,
                    });

                    // Push new blame group
                    if new_group {
                        let author = mem::replace(&mut author, Author::default());
                        let committer = mem::replace(&mut committer, Author::default());
                        let summary = mem::replace(&mut summary, String::new());
                        let previous = previous_commit.take();
                        let blame_lines = mem::replace(&mut blame_lines, Vec::new());

                        blame_groups.push(BlameGroup {
                            author: author.into(),
                            committer: committer.into(),
                            summary,
                            previous,
                            lines: blame_lines,
                        });
                    }

                    state = State::Commit;
                }
            }
        }

        // Final blame group
        if !blame_lines.is_empty() {
            blame_groups.push(BlameGroup {
                author: author.into(),
                committer: committer.into(),
                summary,
                previous: previous_commit,
                lines: blame_lines,
            });
        }

        Ok(Blame {
            groups: blame_groups,
        })
    }
}
