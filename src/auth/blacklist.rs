/*
 * auth/blacklist.rs
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
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub fn build_blacklist(path: &Path) -> Result<HashSet<String>> {
    let file = File::open(path)?;
    let mut buffer = BufReader::new(file);
    let mut blacklist = HashSet::new();

    loop {
        let mut line = String::new();

        if buffer.read_line(&mut line)? == 0 {
            break;
        }

        if line.ends_with('\n') {
            line.pop();
        }

        blacklist.insert(line);
    }

    Ok(blacklist)
}
