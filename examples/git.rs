/*
 * examples/git.rs
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

#![deny(missing_debug_implementations)]

extern crate color_backtrace;
extern crate deepwell;

#[macro_use]
extern crate lazy_static;
extern crate rand;
extern crate tempfile;

use deepwell::{CommitInfo, RevisionStore};
use rand::distributions::Uniform;
use rand::prelude::*;
use std::cmp;
use std::fmt::Write;
use std::ops::{Bound, Range, RangeBounds};
use tempfile::tempdir;

const TEST_SLUGS: [&str; 21] = [
    "main",
    "scp-series",
    "scp-series-2",
    "scp-series-3",
    "scp-series-4",
    "scp-series-5",
    "scp-1000",
    "scp-1111",
    "scp-1730",
    "scp-2000",
    "scp-2003",
    "scp-2790",
    "scp-2998",
    "scp-3000",
    "scp-3220",
    "scp-3999",
    "scp-4000",
    "scp-4002",
    "scp-4220",
    "scp-4444",
    "scp-4999",
];

const TEST_USERNAMES: [&str; 24] = [
    "djkaktus",
    "Roget",
    "ihp",
    "DrClef",
    "DrGears",
    "Tanhony",
    "Zyn",
    "Uncle Nicolini",
    "Kalinin",
    "Jacob Conwell",
    "DrEverettMann",
    "AdminBright",
    "A Random Day",
    "Randomini",
    "The Great Hippo",
    "Captain Kirby",
    "Weryllium",
    "NatVoltaic",
    "DarkStuff",
    "MaliceAforethought",
    "Tufto",
    "DrMagnus",
    "rounderhouse",
    "aismallard",
];

lazy_static! {
    static ref MESSAGE_CHARACTERS: Vec<char> = {
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
            .chars()
            .collect()
    };

    static ref CONTENT_CHARACTERS: Vec<char> = {
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789.,!?'\"   \n\n\n\n\n\n\n\n"
            .chars()
            .collect()
    };
}

#[inline]
fn pick<'a, T, R>(rng: &mut R, items: &'a [T]) -> &'a T
    where R: Rng + ?Sized
{
    items.choose(rng).unwrap()
}

#[inline]
fn pick_str<'a, R, B>(
    rng: &mut R,
    string: &mut String,
    chars: &[char],
    count: usize,
    range: B,
)
    where R: Rng + ?Sized,
          B: RangeBounds<usize> + Clone,
{
    string.replace_range(range.clone(), "");

    let idx = match range.start_bound() {
        Bound::Included(idx) => *idx,
        Bound::Excluded(idx) => *idx + 1,
        Bound::Unbounded => 0,
    };

    for &c in chars.choose_multiple(rng, count) {
        string.insert(idx, c);
    }
}

// Assumes ASCII since getting bounds is annoying, sad.
fn random_range<R>(rng: &mut R, len: usize) -> Range<usize>
    where R: Rng + ?Sized,
{
    let a = rng.gen_range(0, len);
    let b = rng.gen_range(0, len);

    let start = cmp::min(a, b);
    let stop = cmp::max(a, b);
    start..stop
}

fn main() {
    color_backtrace::install();

    // Create revision store
    let directory = tempdir().expect("Unable to create temporary directory");
    let repo = directory.path();
    let store = RevisionStore::new(repo, "example.org");
    store.initial_commit().expect("Unable to create initial commit");

    // Setup shared buffers
    let content_between = Uniform::from(16..8192);
    let mut rng = rand::thread_rng();
    let mut message = String::new();

    // Randomly generate lots of commits
    for _ in 0..500 {
        let slug = pick(&mut rng, TEST_SLUGS.as_ref());
        let username = pick(&mut rng, TEST_USERNAMES.as_ref());

        // Create random message
        write!(&mut message, "Editing file {}: ", slug).unwrap();
        pick_str(&mut rng, &mut message, &MESSAGE_CHARACTERS, 32, ..);

        // Create random content
        let mut content = match store.get_page(slug).expect("Unable to get existing page") {
            Some(bytes) => String::from_utf8(Vec::from(bytes)).expect("Content wasn't UTF-8"),
            None => String::new(),
        };

        let new_content_len = content_between.sample(&mut rng);
        let range = random_range(&mut rng, content.len());
        pick_str(&mut rng, &mut content, &CONTENT_CHARACTERS, new_content_len, range);

        // Commit to repo
        let info = CommitInfo {
            username,
            message: &message,
        };

        store.commit(slug, &content, info).expect("Unable to commit generated data");
    }

    // Randomly delete some pages
    for _ in 0..20 {
        let slug = pick(&mut rng, TEST_SLUGS.as_ref());
        let username = pick(&mut rng, TEST_USERNAMES.as_ref());

        // Create random message
        write!(&mut message, "Deleting file {}: ", slug).unwrap();
        pick_str(&mut rng, &mut message, &MESSAGE_CHARACTERS, 32, ..);

        // Commit to repo
        let info = CommitInfo {
            username,
            message: &message,
        };

        store.remove(slug, info).expect("Unable to commit removed file");
    }
}
