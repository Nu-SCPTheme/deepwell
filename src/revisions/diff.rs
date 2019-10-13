/*
 * revisions/diff.rs
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

use crate::Result;
use git2::Diff as RawDiff;
use std::str;

#[derive(Debug, Clone)]
pub struct DiffLine {
    /// Showing the origin of this diff line.
    ///
    /// * ` ` - Line context
    /// * `+` - Line addition
    /// * `-` - Line deletion
    /// * `=` - Context (end of file)
    /// * `>` - Add (end of file)
    /// * `<` - Remove (end of file)
    /// * `F` - File header
    /// * `H` - Hunk header
    /// * `B` - Line binary
    pub origin: char,

    /// Line number in the old file, or `None` for added line.
    pub old_line_number: Option<u32>,

    /// Line number in the new file, or `None` for deleted line.
    pub new_line_number: Option<u32>,

    /// Number of lines in the contents.
    pub line_count: u32,

    /// The string contents of this line.
    pub contents: String,
}

#[derive(Debug, Clone)]
pub struct Diff {
    /// Total number of inserted lines.
    pub insertions: usize,

    /// Total number of deleted lines.
    pub deletions: usize,

    /// Percent of the file considered to be changed.
    pub percent_changed: f32,

    /// Old page name. Is `None` if the page was created.
    pub old_name: Option<String>,

    /// New page name. Is `None` if the page was deleted.
    pub new_name: Option<String>,

    /// All the lines in this diff.
    pub lines: Vec<DiffLine>,
}

impl Diff {
    pub fn new(raw_diff: RawDiff) -> Result<Self> {
        let stats = raw_diff.stats()?;
        let insertions = stats.insertions();
        let deletions = stats.deletions();

        let mut file_stats = (0.0, None, None);
        let mut lines = Vec::new();

        raw_diff.foreach(
            // Record file stats
            &mut |delta: git2::DiffDelta, percent| -> bool {
                fn path_str(file: git2::DiffFile) -> Option<String> {
                    match file.path_bytes() {
                        Some(bytes) => str::from_utf8(bytes).map(String::from).ok(),
                        None => None,
                    }
                }

                file_stats.0 = percent;
                file_stats.1 = path_str(delta.old_file());
                file_stats.2 = path_str(delta.new_file());

                true
            },
            // No binary stats
            None,
            // No hunk stats
            None,
            // Record line stats
            Some(&mut |_, _, line| -> bool {
                lines.push(DiffLine {
                    origin: line.origin(),
                    old_line_number: line.old_lineno(),
                    new_line_number: line.new_lineno(),
                    line_count: line.num_lines(),
                    contents: String::from_utf8_lossy(line.content()).into_owned(),
                });

                true
            }),
        )?;

        let (percent_changed, old_name, new_name) = file_stats;
        Ok(Diff {
            insertions,
            deletions,
            percent_changed,
            old_name,
            new_name,
            lines,
        })
    }
}
