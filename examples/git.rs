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
use git2::{Repository, Signature};
use rand::Rng;
use rand::distributions::{Alphanumeric, Distribution, Uniform};
use std::fs::File;
use std::mem;
use std::path::Path;
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

fn initial_commit(repo: &Repository) {
    let path = {
        let mut path = repo.path().to_path_buf();
        path.pop();
        path.push(".gitignore");
        path
    };
    println!("Created file {}", path.display());
    let file = File::create(path).expect("Unable to create first file");
    mem::drop(file);

    let signature = Signature::now("DEEPWELL", "noreply@example.com").expect("Unable to create signature");
    let mut index = repo.index().expect("Unable to get git index");
    index.add_path(Path::new(".gitignore")).expect("Add first file");
    let oid = index.write_tree().expect("Write first file to tree");
    let tree = repo.find_tree(oid).expect("Unable to find git tree");
    println!("Staged file");

    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "Initial commit",
        &tree,
        &[],
    ).expect("Unable to create initial commit");
    println!("Created initial commit");
}

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

    let directory = tempdir().expect("Unable to create temporary directory");
    let repo = Repository::init(&directory).expect("Unable to create git repo");
    initial_commit(&repo);
    let store = RevisionStore::new(repo, "example.com");

    let content_between = Uniform::from(16..8192);
    let mut rng = rand::thread_rng();
    let mut message = String::new();
    let mut contents = String::new();

    for _ in 0..10000 {
        let slug = pick(&mut rng, &TEST_SLUGS[..]);
        let username = pick(&mut rng, &TEST_USERNAMES[..]);

        // Create random message
        message.clear();
        for _ in 0..128 {
            message.push(rng.sample(Alphanumeric));
        }

        // Create random content
        contents.clear();
        let content_len = content_between.sample(&mut rng);
        for _ in 0..content_len {
            contents.push(rng.sample(Alphanumeric));
        }

        // Commit to repo
        let info = CommitInfo {
            username,
            message: &message,
        };

        store.commit(slug, &contents, info).expect("Unable to commit generated data");
    }
}
