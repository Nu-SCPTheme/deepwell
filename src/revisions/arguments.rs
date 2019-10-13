/*
 * revisions/arguments.rs
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

use arrayvec::ArrayVec;
use std::ffi::OsStr;

// Sets the maximum number of arguments
type ArgumentArray<'s> = [&'s OsStr; 16];

#[derive(Debug, Clone, Default)]
pub struct Arguments<'s> {
    inner: ArrayVec<ArgumentArray<'s>>,
}

impl<'s> Arguments<'s> {
    pub fn push<S: AsRef<&'s OsStr>>(&mut self, argument: S) {
        self.inner.push(argument.as_ref());
    }

    pub fn push_multiple(&mut self, args: &'s [&str]) {
        args.iter().for_each(|arg| self.inner.push(OsStr::new(arg)));
    }
}

impl<'s> AsRef<[&'s OsStr]> for Arguments<'s> {
    #[inline]
    fn as_ref(&self) -> &[&'s OsStr] {
        &self.inner
    }
}
