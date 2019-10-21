/*
 * macros.rs
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

macro_rules! make_id_type {
    ($name:tt) => {
        use std::borrow::Borrow;

        #[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq)]
        pub struct $name(i64);

        impl Into<i64> for $name {
            #[inline]
            fn into(self) -> i64 {
                self.0
            }
        }

        impl AsRef<i64> for $name {
            #[inline]
            fn as_ref(&self) -> &i64 {
                &self.0
            }
        }

        impl Borrow<i64> for $name {
            #[inline]
            fn borrow(&self) -> &i64 {
                &self.0
            }
        }
    };
}
