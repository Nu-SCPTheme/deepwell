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

use super::{Blame, CommitInfo, GitHash};
use crate::{Error, Result};
use parking_lot::RwLock;
use std::ffi::{OsStr, OsString};
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
    fn get_path(&self, slug: &str, absolute: bool) -> PathBuf {
        let filename = {
            let mut filename = String::new();

            for part in slug.split(':') {
                filename.push_str(part);
                filename.push('$');
            }

            filename.pop();
            filename
        };

        let mut path = PathBuf::new();

        if absolute {
            path.push(&self.repo);
        }

        path.push(&filename);
        path.set_extension("ftml");
        path
    }

    fn read_file(&self, slug: &str) -> Result<Option<Box<[u8]>>> {
        let path = self.get_path(slug, true);
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
        let path = self.get_path(slug, true);
        let mut file = File::create(path)?;
        file.write_all(content)?;
        Ok(())
    }

    fn remove_file(&self, slug: &str) -> Result<Option<()>> {
        let path = self.get_path(slug, true);
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
        format!("--author={} <noreply@{}>", name, self.domain)
    }

    fn arg_message(&self, message: &str) -> String {
        format!("--message={}", message)
    }

    // Process helpers
    fn repo(&self) -> OsString {
        self.repo.as_os_str().to_os_string()
    }

    fn spawn(&self, arguments: &[&OsStr]) -> Result<()> {
        super::spawn(self.repo(), arguments)
    }

    fn spawn_output(&self, arguments: &[&OsStr]) -> Result<Box<[u8]>> {
        super::spawn_output(self.repo(), arguments)
    }

    // Git helpers
    fn get_commit(&self) -> Result<GitHash> {
        let args = arguments!["git", "rev-parse", "--verify", "HEAD",];

        let hex_digest = self.spawn_output(&args)?;
        match GitHash::parse_str(&hex_digest) {
            Some(hash) => Ok(hash),
            None => Err(Error::StaticMsg("unable to parse git hash from output")),
        }
    }

    /// Create the first commit of the repo.
    /// Should only be called on empty repositories.
    #[cold]
    pub fn initial_commit(&self) -> Result<()> {
        let _guard = self.lock.write();

        let args = arguments!["git", "init"];
        self.spawn(&args)?;

        let author = self.arg_author("DEEPWELL");
        let message = self.arg_message("Initial commit");
        let args = arguments!["git", "commit", "--allow-empty", &author, &message];

        self.spawn(&args)?;
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
        check_normal(slug)?;
        self.write_file(slug, content.as_ref())?;

        let path = self.get_path(slug, false);
        let args = arguments!["git", "add", &path];
        self.spawn(&args)?;

        let author = self.arg_author(info.username);
        let message = self.arg_message(info.message);
        let args = arguments!["git", "commit", &author, &message, "--", &path];
        self.spawn(&args)?;

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
        check_normal(slug)?;

        if self.remove_file(slug)?.is_none() {
            return Ok(None);
        }

        let author = self.arg_author(info.username);
        let message = self.arg_message(info.message);
        let path = self.get_path(slug, false);
        let args = arguments!["git", "commit", &author, &message, "--", &path];

        self.spawn(&args)?;
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
        check_normal(slug)?;

        let path = self.get_path(slug, false);
        let spec = format!("{:x}:{}", hash, path.display());
        let args = arguments!["git", "show", "--format=%B", &spec];

        match self.spawn_output(&args) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(Error::CommandFailed(_)) => Ok(None),
            Err(error) => Err(error),
        }
    }

    /// Gets the diff between commits of a particular page.
    /// Returns `None` if the page or commits do not exist.
    pub fn get_diff<S>(&self, slug: S, first: GitHash, second: GitHash) -> Result<Box<[u8]>>
    where
        S: AsRef<str>,
    {
        let _guard = self.lock.read();
        let slug = slug.as_ref();
        let first = format!("{:x}", first);
        let second = format!("{:x}", second);
        check_normal(slug)?;
        let path = self.get_path(slug, false);

        let args = arguments![
            "git",
            "diff",
            "--word-diff=porcelain",
            &first,
            &second,
            "--",
            &path,
        ];
        self.spawn_output(&args)
    }

    /// Gets the blame for a particular page.
    /// Returns `None` if the page does not exist.
    pub fn get_blame<S>(&self, slug: S) -> Result<Option<Blame>>
    where
        S: AsRef<str>,
    {
        let _guard = self.lock.read();
        let slug = slug.as_ref();
        check_normal(slug)?;
        let path = self.get_path(slug, false);

        let args = arguments!["git", "blame", "--porcelain", "--", &path];

        let raw_blame = match self.spawn_output(&args) {
            Ok(bytes) => bytes,
            Err(Error::CommandFailed(_)) => return Ok(None),
            Err(error) => return Err(error),
        };

        let blame = Blame::from_porcelain(&raw_blame)?;
        Ok(Some(blame))
    }
}

fn check_normal(slug: &str) -> Result<()> {
    if is_normal(slug, false) {
        Ok(())
    } else {
        Err(Error::StaticMsg("slug not in wikidot normal form"))
    }
}
