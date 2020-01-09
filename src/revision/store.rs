/*
 * revision/store.rs
 *
 * deepwell - Database management and migrations service
 * Copyright (C) 2019-2020 Ammon Smith
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
use async_std::fs::{self, File};
use async_std::prelude::*;
use async_std::sync::{Mutex, RwLock};
use std::convert::TryFrom;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::str;
use wikidot_normalize::is_normal;

macro_rules! arguments {
    ($($x:expr), *) => {{
        use arrayvec::ArrayVec;

        let mut arguments = ArrayVec::<[&OsStr; 16]>::new();

        $(
            arguments.push(OsStr::new($x));
        )*

        arguments
    }};
    ($($x:expr,)*) => (arguments![$($x),*]);
}

macro_rules! check_normal {
    ($slug:expr) => {
        match check_normal($slug) {
            Ok(_) => (),
            Err(error) => return Err(error),
        }
    };
}

fn check_normal(slug: &str) -> Result<()> {
    trace!("Checking slug for normal form: {}", slug);

    if is_normal(slug, false) {
        Ok(())
    } else {
        Err(Error::StaticMsg("slug not in wikidot normal form"))
    }
}

/// An object that can't be copied or cloned for the `Mutex`.
#[derive(Debug)]
struct RevisionBlock;

/// Represents a git repository to store page contents and their histories.
#[derive(Debug)]
pub struct RevisionStore {
    mutex: Mutex<RevisionBlock>,
    repo: PathBuf,
    domain: RwLock<String>,
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
        let mutex = Mutex::new(RevisionBlock);
        let repo = repo.into();
        let domain = domain.into();

        info!(
            "Creating new revision store for repository {}, domain {}",
            repo.display(),
            domain,
        );

        let domain = RwLock::new(domain);

        RevisionStore {
            mutex,
            repo,
            domain,
        }
    }

    // Filesystem helpers
    fn get_path(&self, slug: &str, absolute: bool) -> PathBuf {
        trace!(
            "Converting slug '{}' to path (absolute: {})",
            slug,
            absolute,
        );

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

    async fn read_file(&self, _guard: &mut RevisionBlock, slug: &str) -> Result<Option<Box<[u8]>>> {
        let path = self.get_path(slug, true);

        debug!("Reading file from {}", path.display());

        let mut file = match File::open(&path).await {
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
        file.read_to_end(&mut content).await?;
        let bytes = content.into_boxed_slice();
        Ok(Some(bytes))
    }

    async fn write_file(
        &self,
        _guard: &mut RevisionBlock,
        slug: &str,
        content: &[u8],
    ) -> Result<()> {
        let path = self.get_path(slug, true);

        debug!("Writing {} bytes to {}", content.len(), path.display());

        let mut file = File::create(path).await?;
        file.write_all(content).await?;
        Ok(())
    }

    async fn remove_file(&self, _guard: &mut RevisionBlock, slug: &str) -> Result<Option<()>> {
        let path = self.get_path(slug, true);

        debug!("Removing file {}", path.display());

        match fs::remove_file(path).await {
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
    async fn arg_author(&self, name: &str) -> String {
        let domain = self.domain.read().await;

        format!("--author={} <noreply@{}>", name, domain)
    }

    fn arg_message(&self, message: &str) -> String {
        format!("--message={}", message)
    }

    // Process helpers
    fn repo(&self) -> OsString {
        self.repo.as_os_str().to_os_string()
    }

    async fn spawn(&self, _guard: &mut RevisionBlock, arguments: &[&OsStr]) -> Result<()> {
        // TODO async-ify
        super::spawn(self.repo(), arguments)
    }

    async fn spawn_output(
        &self,
        _guard: &mut RevisionBlock,
        arguments: &[&OsStr],
    ) -> Result<Box<[u8]>> {
        // TODO async-ify
        super::spawn_output(self.repo(), arguments)
    }

    // Git helpers
    async fn get_commit(&self, guard: &mut RevisionBlock) -> Result<GitHash> {
        debug!("Getting current HEAD commit");

        let args = arguments!["git", "rev-parse", "--verify", "HEAD"];

        let digest_bytes = self.spawn_output(guard, &args).await?;
        let digest = str::from_utf8(&digest_bytes)
            .map_err(|_| Error::StaticMsg("git hash wasn't valid UTF-8"))?;

        let hash = GitHash::try_from(digest)
            .map_err(|_| Error::StaticMsg("unable to parse git hash from output"))?;

        Ok(hash)
    }

    #[cfg(test)]
    async fn check_clean(&self, guard: &mut RevisionBlock) {
        debug!("Checking if repository is clean");

        let args = arguments!["git", "status", "--porcelain"];
        let output = self
            .spawn_output(guard, &args)
            .await
            .expect("Unable to get git status");

        if !output.is_empty() {
            panic!(
                "Git repository is not clean:\n{}",
                String::from_utf8_lossy(&output),
            );
        }
    }

    #[cfg(not(test))]
    async fn check_clean(&self, _guard: &mut RevisionBlock) {}

    /// Create the first commit of the repo.
    /// Should only be called on empty repositories.
    #[cold]
    pub async fn initial_commit(&self) -> Result<()> {
        info!("Initializing new git repository");

        let guard = &mut self.mutex.lock().await;
        let args = arguments!["git", "init"];
        self.spawn(guard, &args).await?;

        let author = self.arg_author("DEEPWELL").await;
        let message = self.arg_message("Initial commit");
        let args = arguments!["git", "commit", "--allow-empty", &author, &message];

        self.spawn(guard, &args).await?;
        self.check_clean(guard).await;

        Ok(())
    }

    /// For the given slug, create or edit a page to have the specified contents.
    pub async fn commit(
        &self,
        slug: &str,
        content: Option<&[u8]>,
        info: CommitInfo<'_>,
    ) -> Result<GitHash> {
        info!(
            "Committing file changes for slug '{}' ({} bytes)",
            slug,
            content.map(|b| b.len()).unwrap_or(0),
        );

        check_normal!(slug);
        let guard = &mut self.mutex.lock().await;

        if let Some(content) = content {
            self.write_file(guard, slug, content).await?;
        }

        let path = self.get_path(slug, false);
        let args = arguments!["git", "add", &path];
        self.spawn(guard, &args).await?;

        let author = self.arg_author(info.username).await;
        let message = self.arg_message(info.message);
        let args = arguments![
            "git",
            "commit",
            "--allow-empty",
            &author,
            &message,
            "--",
            &path,
        ];
        self.spawn(guard, &args).await?;

        let commit = self.get_commit(guard).await?;
        self.check_clean(guard).await;

        Ok(commit)
    }

    /// Creates an empty commit.
    pub async fn empty_commit(&self, info: CommitInfo<'_>) -> Result<GitHash> {
        info!("Creating empty commit");

        let guard = &mut self.mutex.lock().await;
        let author = self.arg_author(info.username).await;
        let message = self.arg_message(info.message);

        let args = arguments!["git", "commit", "--allow-empty", &author, &message];
        self.spawn(guard, &args).await?;

        let commit = self.get_commit(guard).await?;
        self.check_clean(guard).await;

        Ok(commit)
    }

    /// Renames the given page in the repository.
    pub async fn rename(
        &self,
        old_slug: &str,
        new_slug: &str,
        info: CommitInfo<'_>,
    ) -> Result<GitHash> {
        info!("Renaming file for slug '{}' -> '{}'", old_slug, new_slug);

        check_normal!(old_slug);
        check_normal!(new_slug);
        let guard = &mut self.mutex.lock().await;

        let new_path = self.get_path(new_slug, true);
        if new_path.exists() {
            return Err(Error::PageExists);
        }

        let old_path = self.get_path(old_slug, false);
        let new_path = self.get_path(new_slug, false);
        let args = arguments!["git", "mv", "--", &old_path, &new_path];
        self.spawn(guard, &args).await?;

        let author = self.arg_author(info.username).await;
        let message = self.arg_message(info.message);
        let args = arguments!["git", "commit", &author, &message, "--", &old_path, &new_path];
        self.spawn(guard, &args).await?;

        let commit = self.get_commit(guard).await?;
        self.check_clean(guard).await;

        Ok(commit)
    }

    /// Remove the given page from the repository.
    /// Returns `None` if the page does not exist.
    pub async fn remove(&self, slug: &str, info: CommitInfo<'_>) -> Result<Option<GitHash>> {
        info!("Removing file for slug '{}' (info: {:?})", slug, info);

        check_normal!(slug);
        let guard = &mut self.mutex.lock().await;

        let removed = self.remove_file(guard, slug).await?;
        if removed.is_none() {
            return Ok(None);
        }

        let author = self.arg_author(info.username).await;
        let message = self.arg_message(info.message);
        let path = self.get_path(slug, false);
        let args = arguments!["git", "commit", &author, &message, "--", &path];

        self.spawn(guard, &args).await?;

        let commit = self.get_commit(guard).await.map(Some)?;
        self.check_clean(guard).await;

        Ok(commit)
    }

    /// Restores the given page from the given hash.
    /// This is performed by committing the page as it
    /// existed at the time.
    ///
    /// This is equivalent to Wikidot's "revert" functionality,
    /// but potentially across page boundaries. Not to be confused
    /// with git's notion of a "revert".
    pub async fn restore(
        &self,
        slug: &str,
        old_slug: &str,
        hash: &GitHash,
        info: CommitInfo<'_>,
    ) -> Result<GitHash> {
        info!(
            "Restoring file '{}' from {} onto '{}' (info: {:?})",
            old_slug, hash, slug, info,
        );

        check_normal!(slug);
        check_normal!(old_slug);

        let guard = &mut self.mutex.lock().await;

        // Get old page content
        let content = {
            let path = self.get_path(old_slug, false);
            let spec = format!("{}:{}", hash, path.display());
            let args = arguments!["git", "show", "--format=%B", &spec];

            match self.spawn_output(guard, &args).await {
                Ok(bytes) => Ok(bytes),
                Err(Error::CommandFailed(_)) => Err(Error::PageNotFound),
                Err(error) => Err(error),
            }
        }?;

        // Write and commit contents
        self.write_file(guard, slug, &content).await?;

        let path = self.get_path(slug, false);
        let args = arguments!["git", "add", &path];
        self.spawn(guard, &args).await?;

        let author = self.arg_author(info.username).await;
        let message = self.arg_message(info.message);
        let args = arguments![
            "git",
            "commit",
            "--allow-empty",
            &author,
            &message,
            "--",
            &path,
        ];
        self.spawn(guard, &args).await?;

        let commit = self.get_commit(guard).await?;
        self.check_clean(guard).await;

        Ok(commit)
    }

    /// Reverts the given commit.
    /// This performs a standard `git revert` but edits the message.
    ///
    /// This is distinct from Wikidot's notion of a "revert", which is
    /// why it is called "undo" throughout the code.
    pub async fn undo(&self, hash: &GitHash, info: CommitInfo<'_>) -> Result<GitHash> {
        info!("Undoing commit {} (info: {:?})", hash, info);

        let guard = &mut self.mutex.lock().await;

        // Perform the revert
        let args = arguments!["git", "revert", "--no-edit", hash];
        self.spawn(guard, &args).await?;

        // Edit the commit message
        let author = self.arg_author(info.username).await;
        let message = self.arg_message(info.message);
        let args = arguments!["git", "commit", "--amend", &author, &message,];
        self.spawn(guard, &args).await?;

        let commit = self.get_commit(guard).await?;
        self.check_clean(guard).await;

        Ok(commit)
    }

    /// Gets the current version of a page.
    /// Returns `None` if the page does not exist.
    pub async fn get_page(&self, slug: &str) -> Result<Option<Box<[u8]>>> {
        info!("Getting page content for slug '{}'", slug);

        check_normal!(slug);
        let guard = &mut self.mutex.lock().await;

        let contents = self.read_file(guard, slug).await?;
        self.check_clean(guard).await;

        Ok(contents)
    }

    /// Gets the version of a page at the specified commit.
    /// Returns `None` if the page did not at exist at the time.
    pub async fn get_page_version(&self, slug: &str, hash: &GitHash) -> Result<Option<Box<[u8]>>> {
        info!(
            "Getting page content for slug '{}' at commit {}",
            slug, hash,
        );

        check_normal!(slug);
        let guard = &mut self.mutex.lock().await;

        let path = self.get_path(slug, false);
        let spec = format!("{}:{}", hash, path.display());
        let args = arguments!["git", "show", "--format=%B", &spec];

        let result = match self.spawn_output(guard, &args).await {
            Ok(bytes) => Ok(Some(bytes)),
            Err(Error::CommandFailed(_)) => Ok(None),
            Err(error) => Err(error),
        };

        self.check_clean(guard).await;
        result
    }

    /// Gets the diff between commits of a particular page.
    /// Returns `None` if the page or commits do not exist.
    pub async fn get_diff(
        &self,
        slug: &str,
        first: &GitHash,
        second: &GitHash,
    ) -> Result<Box<[u8]>> {
        info!(
            "Getting diff for slug '{}' between {}..{}",
            slug, first, second,
        );

        check_normal!(slug);
        let guard = &mut self.mutex.lock().await;
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

        let diff = self.spawn_output(guard, &args).await?;
        self.check_clean(guard).await;

        Ok(diff)
    }

    /// Gets the blame for a particular page.
    /// Returns `None` if the page does not exist.
    pub async fn get_blame(&self, slug: &str, hash: Option<&GitHash>) -> Result<Option<Blame>> {
        info!("Getting blame for slug '{}'", slug);

        check_normal!(slug);
        let guard = &mut self.mutex.lock().await;
        let path = self.get_path(slug, false);

        let args = match hash {
            Some(ref hash) => arguments!["git", "blame", "--porcelain", hash, "--", &path],
            None => arguments!["git", "blame", "--porcelain", "--", &path],
        };

        let raw_blame = match self.spawn_output(guard, &args).await {
            Ok(bytes) => bytes,
            Err(Error::CommandFailed(_)) => return Ok(None),
            Err(error) => return Err(error),
        };

        let blame = Blame::from_porcelain(&raw_blame)?;
        self.check_clean(guard).await;

        Ok(Some(blame))
    }

    /// Sets the domain to a different value.
    pub async fn set_domain(&self, new_domain: &str) {
        trace!("Acquiring domain write lock to change: {}", new_domain);

        let mut guard = self.domain.write().await;
        guard.clear();
        guard.push_str(new_domain);
    }
}
