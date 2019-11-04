/*
 * revision/git_hash.rs
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

use arrayvec::ArrayString;
use crate::StdResult;
use regex::Regex;
use std::borrow::Borrow;
use std::convert::TryFrom;
use std::ffi::OsStr;
use std::fmt::{self, Debug, Display};
use std::str;

lazy_static! {
    static ref GIT_HASH_REGEX: Regex = Regex::new(r"[a-f0-9]{40}").unwrap();
}

#[derive(Clone, PartialEq, Eq)]
pub struct GitHash(Box<ArrayString<[u8; 40]>>);

impl GitHash {
    pub fn from_checked<B>(hash: B) -> Self
    where
        B: Borrow<str>,
    {
        let hash = hash.borrow();

        Self::try_from(hash).expect("Invalid git hash")
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for GitHash {
    type Error = ();

    fn try_from(hash: &str) -> StdResult<Self, ()> {
        let hash = hash.trim();

        if GIT_HASH_REGEX.is_match(hash) {
            let arrstr = ArrayString::from(hash).unwrap();

            Ok(GitHash(Box::new(arrstr)))
        } else {
            Err(())
        }
    }
}

impl AsRef<str> for GitHash {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<OsStr> for GitHash {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        OsStr::new(self.as_str())
    }
}

impl Debug for GitHash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("GitHash").field(&self.0).finish()
    }
}

impl Display for GitHash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self)
    }
}
