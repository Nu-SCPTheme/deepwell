/*
 * revisions/mod.rs
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

use git2::Repository;
use std::fmt::{self, Debug};

pub struct RevisionStore {
    repo: Repository,
}

impl RevisionStore {
    #[inline]
    pub fn new(repo: Repository) -> Self {
        RevisionStore { repo }
    }
}

impl Debug for RevisionStore {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let repo = format!(
            "git2::Repository {{ path: {}, .. }}",
            self.repo.path().display(),
        );

        f.debug_struct("RevisionStore")
            .field("repo", &repo)
            .finish()
    }
}
