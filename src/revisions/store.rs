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

use super::{CommitInfo, GitHash};
use crate::{Error, Result};
use git2::{Commit, ObjectType, Repository, RepositoryState, Signature};
use parking_lot::Mutex;
use std::fmt::{self, Debug};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use wikidot_normalize::{is_normal, normalize};

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

    fn write_file(&self, repo_root: &Path, slug: &str, contents: &[u8]) -> Result<PathBuf> {
        if !is_normal(slug, false) {
            return Err(Error::StaticMsg("slug not in wikidot normal form"));
        }

        let path = {
            let mut path = PathBuf::new();
            path.push(repo_root);
            path.push(slug);
            path.set_extension(FILE_EXTENSION);
            path
        };

        let mut file = File::create(&path)?;
        file.write_all(contents)?;

        Ok(path)
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

            Signature::now("DEEPWELL", &email)?
        };

        Ok((author, committer))
    }

    /// For the given slug, create a revision with the new contents provided.
    pub fn commit(&self, slug: &str, contents: &[u8], info: CommitInfo) -> Result<GitHash> {
        let repo = self.repo.lock();
        check_repo!(repo);

        // Stage file changes
        let mut index = repo.index()?;
        let path = self.write_file(repo.path(), slug, contents)?;
        index.add_path(&path)?;
        let oid = index.write_tree()?;

        // Create commit
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

        Ok(GitHash::from(commit))
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
