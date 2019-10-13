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
extern crate git2;
extern crate rand;
extern crate tempfile;

use deepwell::{CommitInfo, RevisionStore};
use git2::Repository;
use rand::Rng;
use rand::distributions::{Alphanumeric, Distribution, Uniform};
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

fn pick<'a, T, R>(rng: &mut R, items: &'a [T]) -> &'a T
    where R: Rng + ?Sized
{
    let len = items.len();
    let between = Uniform::from(0..len);
    let idx = between.sample(rng);
    return &items[idx];
}

fn main() {
    color_backtrace::install();

    // Create revision store
    let directory = tempdir().expect("Unable to create temporary directory");
    let repo = Repository::init(&directory).expect("Unable to create git repo");
    let store = RevisionStore::new(repo, "example.org");
    store.initial_commit().expect("Unable to create initial commit");

    // Setup shared buffers
    let content_between = Uniform::from(16..8192);
    let mut rng = rand::thread_rng();
    let mut message = String::new();
    let mut content = String::new();

    // Randomly generate lots of commits
    for _ in 0..100 {
        let slug = pick(&mut rng, &TEST_SLUGS[..]);
        let username = pick(&mut rng, &TEST_USERNAMES[..]);

        // Create random message
        message.clear();
        for _ in 0..128 {
            message.push(rng.sample(Alphanumeric));
        }

        // Create random content
        content.clear();
        let content_len = content_between.sample(&mut rng);
        for _ in 0..content_len {
            content.push(rng.sample(Alphanumeric));
        }
        content.push('\n');

        // Commit to repo
        let info = CommitInfo {
            username,
            message: &message,
        };

        store.commit(slug, &content, info).expect("Unable to commit generated data");
    }

    // Randomly delete some pages
    message.clear();
    message.push_str("Deleting page");

    for _ in 0..5 {
        let slug = pick(&mut rng, &TEST_SLUGS[..]);
        let username = pick(&mut rng, &TEST_USERNAMES[..]);

        // Commit to repo
        let info = CommitInfo {
            username,
            message: &message,
        };

        store.remove(slug, info).expect("Unable to commit removed file");
    }
}
