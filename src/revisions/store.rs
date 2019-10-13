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

use super::{CommitInfo, Diff, GitHash};
use crate::{Error, Result};
use git2::{Commit, ObjectType, Oid, Repository, RepositoryState, Signature};
use parking_lot::Mutex;
use std::fmt::{self, Debug};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use wikidot_normalize::{is_normal, normalize};

const SYSTEM_USER: &str = "DEEPWELL";
const FILE_EXTENSION: &str = "ftml";

macro_rules! check_repo {
    ($repo:expr) => {
        match $repo.state() {
            RepositoryState::Clean => (),
            _ => return Err(Error::StaticMsg("repository is not in a clean state")),
        }
    };
}

/// Represents a git repository to store page contents and their histories.
pub struct RevisionStore {
    repo: Mutex<Repository>,
    domain: String,
}

impl RevisionStore {
    /// Creates a new revision store using the given `git2::Repository` and domain name.
    ///
    /// The domain name should not be prefixed with a protocol such as `https://` but does
    /// permit subdomains.
    #[inline]
    pub fn new<I: Into<String>>(repo: Repository, domain: I) -> Self {
        RevisionStore {
            repo: Mutex::new(repo),
            domain: domain.into(),
        }
    }

    fn path(root: Option<&Path>, slug: &str) -> Result<PathBuf> {
        if !is_normal(slug, false) {
            return Err(Error::StaticMsg("slug not in wikidot normal form"));
        }

        Ok(Self::unchecked_path(root, slug))
    }

    fn unchecked_path(root: Option<&Path>, slug: &str) -> PathBuf {
        let mut path = PathBuf::new();

        if let Some(root) = root {
            // If passed, make an absolute path
            path.push(root);

            // Remove the .git directory
            path.pop();
        }

        path.push(slug);
        path.set_extension(FILE_EXTENSION);
        path
    }

    fn write_file(&self, path: &Path, contents: &[u8]) -> Result<()> {
        let mut file = File::create(path)?;
        file.write_all(contents)?;

        Ok(())
    }

    fn find_last_commit(repo: &Repository) -> Result<Commit> {
        let head = repo.head()?.resolve()?;
        let obj = head.peel(ObjectType::Commit)?;
        obj.into_commit()
            .map_err(|_| Error::StaticMsg("repository has no commits"))
    }

    fn get_signatures(&self, info: &CommitInfo) -> Result<(Signature, Signature)> {
        let mut email = String::new();

        let author = {
            email.push_str(info.username);
            normalize(&mut email);
            email.push_str(".user@");
            email.push_str(&self.domain);

            Signature::now(info.username, &email)?
        };

        let committer = {
            email.clear();
            email.push_str("system@");
            email.push_str(&self.domain);

            Signature::now(SYSTEM_USER, &email)?
        };

        Ok((author, committer))
    }

    fn raw_commit(&self, repo: &Repository, path: &Path, info: CommitInfo) -> Result<Oid> {
        // Stage file changes
        let mut index = repo.index()?;
        index.add_path(&path)?;
        let oid = index.write_tree()?;

        // Commit to branch
        let parent = Self::find_last_commit(&repo)?;
        let tree = repo.find_tree(oid)?;
        let (author, committer) = self.get_signatures(&info)?;
        let commit = repo.commit(
            Some("HEAD"),
            &author,
            &committer,
            info.message,
            &tree,
            &[&parent],
        )?;

        Ok(commit)
    }

    /// Create the first commit of the repo.
    /// Should only be called on empty repositories.
    #[cold]
    pub fn initial_commit(&self) -> Result<GitHash> {
        let repo = self.repo.lock();
        check_repo!(repo);

        let path = Self::unchecked_path(Some(repo.path()), ".gitignore");
        self.write_file(&path, &[])?;

        // Stage file changes
        let mut index = repo.index()?;
        index.add_path(&path)?;
        let oid = index.write_tree()?;

        // Create first commit
        let tree = repo.find_tree(oid)?;
        let email = format!("system@{}", self.domain);
        let signature = Signature::now(SYSTEM_USER, &email)?;
        let commit = repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Initial commit",
            &tree,
            &[],
        )?;

        Ok(GitHash::from(commit))
    }

    /// For the given slug, create or edit a page to have the specified contents.
    pub fn commit<S, B>(&self, slug: S, contents: B, info: CommitInfo) -> Result<GitHash>
    where
        S: AsRef<str>,
        B: AsRef<[u8]>,
    {
        let repo = self.repo.lock();
        check_repo!(repo);

        let path = Self::path(Some(repo.path()), slug.as_ref())?;
        self.write_file(&path, contents.as_ref())?;
        let commit_oid = self.raw_commit(&repo, &path, info)?;

        Ok(GitHash::from(commit_oid))
    }

    /// Remove the given page from the repository.
    pub fn remove<S>(&self, slug: S, info: CommitInfo) -> Result<GitHash>
    where
        S: AsRef<str>,
    {
        let repo = self.repo.lock();
        check_repo!(repo);

        let path = Self::path(Some(repo.path()), slug.as_ref())?;
        fs::remove_file(&path)?;
        let commit_oid = self.raw_commit(&repo, &path, info)?;

        Ok(GitHash::from(commit_oid))
    }

    /// Gets the current version of a page.
    /// Returns `None` if the page does not exist.
    pub fn get_page<S>(&self, slug: S) -> Result<Option<Box<[u8]>>>
    where
        S: AsRef<str>,
    {
        let repo = self.repo.lock();
        check_repo!(repo);

        let path = Self::path(Some(repo.path()), slug.as_ref())?;
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

        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;
        let bytes = contents.into_boxed_slice();
        Ok(Some(bytes))
    }

    fn find_commit(repo: &Repository, hash: GitHash) -> Result<Option<Commit>> {
        let oid = Oid::from_bytes(hash.as_ref())?;

        match repo.find_commit(oid) {
            Ok(commit) => Ok(Some(commit)),
            Err(error) => {
                use git2::ErrorCode;

                match error.code() {
                    ErrorCode::NotFound => Ok(None),
                    _ => Err(Error::from(error)),
                }
            }
        }
    }

    /// Gets the version of a page at the specified commit.
    /// Returns `None` if the page did not at exist at the time.
    pub fn get_page_version<S>(&self, slug: S, hash: GitHash) -> Result<Option<Box<[u8]>>>
    where
        S: AsRef<str>,
    {
        let repo = self.repo.lock();
        check_repo!(repo);

        let commit = match Self::find_commit(&repo, hash)? {
            Some(commit) => commit,
            None => return Ok(None),
        };

        let tree = commit.tree()?;
        let path = Self::path(None, slug.as_ref())?;
        let entry = tree.get_path(&path)?;
        let obj = entry.to_object(&repo)?;
        let blob = obj
            .into_blob()
            .map_err(|_| Error::StaticMsg("tree object is not a blob"))?;

        let bytes = blob.content().to_vec().into_boxed_slice();
        Ok(Some(bytes))
    }

    /// Gets the diff between commits of a particular page.
    /// Returns `None` if the page or commits do not exist.
    pub fn get_diff<S>(&self, slug: S, first: GitHash, second: GitHash) -> Result<Option<Diff>>
    where
        S: AsRef<str>,
    {
        use git2::DiffOptions;

        let repo = self.repo.lock();
        check_repo!(repo);

        let first_commit = Self::find_commit(&repo, first)?;
        let second_commit = Self::find_commit(&repo, second)?;

        let (first_tree, second_tree) = match (first_commit, second_commit) {
            (Some(first_commit), Some(second_commit)) => {
                let first_tree = first_commit.tree()?;
                let second_tree = second_commit.tree()?;

                (first_tree, second_tree)
            }
            _ => return Ok(None),
        };

        let path = Self::path(None, slug.as_ref())?;
        let raw_diff = repo.diff_tree_to_tree(
            Some(&first_tree),
            Some(&second_tree),
            Some(
                DiffOptions::new()
                    .minimal(true)
                    .force_text(true)
                    .pathspec(path),
            ),
        )?;

        let diff = Diff::new(raw_diff)?;
        Ok(Some(diff))
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

impl Debug for RevisionStore {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let repo = format!(
            "git2::Repository {{ path: {}, .. }}",
            self.repo.lock().path().display(),
        );

        f.debug_struct("RevisionStore")
            .field("repo", &repo)
            .field("domain", &self.domain)
            .finish()
    }
}
