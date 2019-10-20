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
use rand::prelude::*;
use std::cmp;
use std::fmt::Write;
use std::ops::{Bound, Range, RangeBounds};
use tempfile::tempdir;

const TEST_SLUGS: [&str; 89] = [
    "main",
    "scp-001",
    "scp-001-ex",
    "scp-049",
    "scp-055",
    "scp-160",
    "scp-173",
    "scp-280-jp",
    "scp-426",
    "scp-488-jp",
    "scp-579",
    "scp-682",
    "scp-882",
    "scp-1000",
    "scp-1111",
    "scp-1474",
    "scp-1730",
    "scp-2000",
    "scp-2003",
    "scp-2111",
    "scp-2316",
    "scp-2521",
    "scp-2669",
    "scp-2718",
    "scp-2719",
    "scp-2747",
    "scp-2790",
    "scp-2998",
    "scp-3000",
    "scp-3003",
    "scp-3008",
    "scp-3009",
    "scp-3031",
    "scp-3095",
    "scp-3125",
    "scp-3133",
    "scp-3160",
    "scp-3211",
    "scp-3220",
    "scp-3287",
    "scp-3362",
    "scp-3393",
    "scp-3396",
    "scp-3448",
    "scp-3597",
    "scp-3731",
    "scp-3733",
    "scp-3838",
    "scp-3908",
    "scp-3942",
    "scp-3999",
    "scp-4000",
    "scp-4002",
    "scp-4004",
    "scp-4004",
    "scp-4006",
    "scp-4028",
    "scp-4096",
    "scp-4144",
    "scp-4163",
    "scp-4220",
    "scp-4224",
    "scp-4242",
    "scp-4316",
    "scp-4322",
    "scp-4339",
    "scp-4339",
    "scp-4355",
    "scp-4406",
    "scp-4444",
    "scp-4447",
    "scp-4455",
    "scp-4504",
    "scp-4514",
    "scp-4560",
    "scp-4603",
    "scp-4636",
    "scp-4662",
    "scp-4727",
    "scp-4833",
    "scp-4838",
    "scp-4853",
    "scp-4882",
    "scp-4999",
    "scp-series",
    "scp-series-2",
    "scp-series-3",
    "scp-series-4",
    "scp-series-5",
];

const TEST_USERNAMES: [&str; 25] = [
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
    "not_a_seagull",
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
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789.,!?'\" \n\n\n\n"
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
    let start = rng.gen_range(0, len);
    let size = rng.gen_range(3, 64);
    let end = start + cmp::min(size, len - start);

    start..end
}

fn main() {
    color_backtrace::install();

    // Create revision store
    let directory = tempdir().expect("Unable to create temporary directory");
    let repo = directory.path();
    let store = RevisionStore::new(repo, "example.org");
    store.initial_commit().expect("Unable to create initial commit");
    println!("Starting generation...");

    // Setup shared buffers
    let mut rng = rand::thread_rng();
    let mut message = String::new();

    // Randomly generate lots of commits
    for _ in 0..500 {
        let slug = pick(&mut rng, TEST_SLUGS.as_ref());
        let username = pick(&mut rng, TEST_USERNAMES.as_ref());

        // Create random message
        message.clear();
        write!(&mut message, "Editing file {}: ", slug).unwrap();
        let range = message.len()..;
        pick_str(&mut rng, &mut message, &MESSAGE_CHARACTERS, 32, range);

        // Create random content
        let mut content = match store.get_page(slug).expect("Unable to get existing page") {
            Some(bytes) => {
                let bytes = Vec::from(bytes);
                String::from_utf8(bytes).expect("Content wasn't UTF-8")
            }
            None => {
                let mut content = String::new();
                let len = rng.gen_range(4096, 100_000);
                pick_str(&mut rng, &mut content, &CONTENT_CHARACTERS, len, ..);
                content
            }
        };

        let content_len = rng.gen_range(8, 128);
        let range = random_range(&mut rng, content.len());
        pick_str(&mut rng, &mut content, &CONTENT_CHARACTERS, content_len, range);

        // Commit to repo
        let info = CommitInfo {
            username,
            message: &message,
        };

        store.commit(slug, &content, info).expect("Unable to commit generated data");
    }

    // Randomly delete some pages
    for _ in 0..5 {
        let slug = pick(&mut rng, TEST_SLUGS.as_ref());
        let username = pick(&mut rng, TEST_USERNAMES.as_ref());

        // Create random message
        message.clear();
        write!(&mut message, "Deleting file {}: ", slug).unwrap();
        let range = message.len()..;
        pick_str(&mut rng, &mut message, &MESSAGE_CHARACTERS, 32, range);

        // Commit to repo
        let info = CommitInfo {
            username,
            message: &message,
        };

        store.remove(slug, info).expect("Unable to commit removed file");
    }
}
