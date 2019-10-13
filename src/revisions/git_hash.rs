/*
 * revisions/git_hash.rs
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

use std::fmt::{self, UpperHex, LowerHex};
use std::iter::FromIterator;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct GitHash([u8; 20]);

impl From<[u8; 20]> for GitHash {
    #[inline]
    fn from(hash: [u8; 20]) -> Self {
        GitHash(hash)
    }
}

impl From<&[u8]> for GitHash {
    fn from(bytes: &[u8]) -> Self {
        let mut hash = [0; 20];
        let slice = &mut hash[..];
        slice.copy_from_slice(bytes);
        GitHash(hash)
    }
}

impl AsRef<[u8; 20]> for GitHash {
    #[inline]
    fn as_ref(&self) -> &[u8; 20] {
        &self.0
    }
}

impl AsRef<[u8]> for GitHash {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl LowerHex for GitHash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }

        Ok(())
    }
}

impl UpperHex for GitHash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{:02X}", byte)?;
        }

        Ok(())
    }
}
