/*
 * revisions/store.rs
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

use super::{spawn, spawn_output, CommitInfo, GitHash};
use crate::{Error, Result};
use parking_lot::RwLock;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use wikidot_normalize::is_normal;

macro_rules! arguments {
    ($($x:expr), *) => {{
        use arrayvec::ArrayVec;
        use std::ffi::OsStr;

        let mut arguments = ArrayVec::<[&OsStr; 16]>::new();

        $(
            arguments.push(OsStr::new($x));
        )*

        arguments
    }};
    ($($x:expr,)*) => (arguments![$($x),*]);
}

/// Represents a git repository to store page contents and their histories.
#[derive(Debug)]
pub struct RevisionStore {
    lock: RwLock<()>,
    repo: PathBuf,
    domain: String,
}

impl RevisionStore {
    /// Creates a new revision store using the given `git2::Repository` and domain name.
    ///
    /// The domain name should not be prefixed with a protocol such as `https://` but does
    /// permit subdomains.
    #[inline]
    pub fn new<P, S>(repo: P, domain: S) -> Self
    where
        P: Into<PathBuf>,
        S: Into<String>,
    {
        RevisionStore {
            lock: RwLock::new(()),
            repo: repo.into(),
            domain: domain.into(),
        }
    }

    // Filesystem helpers
    fn read_file(&self, slug: &str) -> Result<Option<Box<[u8]>>> {
        let path = self.repo.join(slug);
        let mut file = match File::open(&path) {
            Ok(file) => file,
            Err(error) => {
                use std::io::ErrorKind;

                return match error.kind() {
                    ErrorKind::NotFound => Ok(None),
                    _ => Err(Error::from(error)),
                };
            }
        };

        let mut content = Vec::new();
        file.read_to_end(&mut content)?;
        let bytes = content.into_boxed_slice();
        Ok(Some(bytes))
    }

    fn write_file(&self, slug: &str, content: &[u8]) -> Result<()> {
        let path = self.repo.join(slug);
        let mut file = File::create(path)?;
        file.write_all(content)?;
        Ok(())
    }

    fn remove_file(&self, slug: &str) -> Result<Option<()>> {
        let path = self.repo.join(slug);
        match fs::remove_file(path) {
            Ok(_) => (),
            Err(error) => {
                use std::io::ErrorKind;

                return match error.kind() {
                    ErrorKind::NotFound => Ok(None),
                    _ => Err(Error::from(error)),
                };
            }
        }

        Ok(Some(()))
    }

    // Argument helpers
    fn arg_author(&self, name: &str) -> String {
        format!("{} <noreply@{}>", name, self.domain)
    }

    fn arg_message(&self, message: &str) -> String {
        format!("--message={}", message)
    }

    // Git helpers
    fn get_commit(&self) -> Result<GitHash> {
        let args = arguments!["git", "rev-parse", "--verify", "HEAD",];

        let hex_digest = spawn_output(&args)?;
        match GitHash::from_str(&hex_digest) {
            Some(hash) => Ok(hash),
            None => Err(Error::StaticMsg("unable to parse git hash from output")),
        }
    }

    /// Create the first commit of the repo.
    /// Should only be called on empty repositories.
    #[cold]
    pub fn initial_commit(&self) -> Result<()> {
        let _guard = self.lock.write();

        let author = self.arg_author("DEEPWELL");
        let message = self.arg_message("Initial commit");
        let args = arguments!["git", "commit", "--allow-empty", &author, &message];

        spawn(&args)?;

        Ok(())
    }

    /// For the given slug, create or edit a page to have the specified contents.
    pub fn commit<S, B>(&self, slug: S, content: B, info: CommitInfo) -> Result<GitHash>
    where
        S: AsRef<str>,
        B: AsRef<[u8]>,
    {
        let _guard = self.lock.write();
        let slug = slug.as_ref();
        self.write_file(slug, content.as_ref())?;

        let author = self.arg_author(info.username);
        let message = self.arg_message(info.message);
        let args = arguments!["git", "commit", &author, &message, "--", slug];

        spawn(&args)?;
        self.get_commit()
    }

    /// Remove the given page from the repository.
    /// Returns `None` if the page does not exist.
    pub fn remove<S>(&self, slug: S, info: CommitInfo) -> Result<Option<GitHash>>
    where
        S: AsRef<str>,
    {
        let _guard = self.lock.write();
        let slug = slug.as_ref();

        if let None = self.remove_file(slug)? {
            return Ok(None);
        }

        let author = self.arg_author(info.username);
        let message = self.arg_message(info.message);
        let args = arguments!["git", "commit", &author, &message, "--", slug];

        spawn(&args)?;
        self.get_commit().map(Some)
    }

    /// Gets the current version of a page.
    /// Returns `None` if the page does not exist.
    pub fn get_page<S>(&self, slug: S) -> Result<Option<Box<[u8]>>>
    where
        S: AsRef<str>,
    {
        let _guard = self.lock.read();
        let slug = slug.as_ref();

        check_normal(slug)?;
        self.read_file(slug)
    }

    /// Gets the version of a page at the specified commit.
    /// Returns `None` if the page did not at exist at the time.
    pub fn get_page_version<S>(&self, slug: S, hash: GitHash) -> Result<Option<Box<[u8]>>>
    where
        S: AsRef<str>,
    {
        let _guard = self.lock.read();
        let slug = slug.as_ref();

        let spec = format!("{:x}:{}", hash, slug);
        let args = arguments!["git", "show", "--format=%B", &spec,];

        match spawn_output(&args) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(Error::CommandFailed(_)) => Ok(None),
            Err(error) => Err(error),
        }
    }

    /// Gets the diff between commits of a particular page.
    /// Returns `None` if the page or commits do not exist.
    pub fn get_diff<S>(&self, _slug: S, _first: GitHash, _second: GitHash) -> Result<Option<()>>
    where
        S: AsRef<str>,
    {
        Err(Error::StaticMsg("not implemented yet"))
    }

    /// Gets the blame for a particular page.
    /// Returns `None` if the page does not exist.
    pub fn get_blame<S>(&self, _slug: S) -> Result<Option<()>>
    where
        S: AsRef<str>,
    {
        Err(Error::StaticMsg("not implemented yet"))
    }
}

fn check_normal(slug: &str) -> Result<()> {
    if is_normal(slug, false) {
        Ok(())
    } else {
        Err(Error::StaticMsg("slug not in wikidot normal form"))
    }
}
