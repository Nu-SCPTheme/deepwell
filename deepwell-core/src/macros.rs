/*
 * macros.rs
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

macro_rules! make_id_type {
    ($module:ident, $name:tt) => {
        mod $module {
            use diesel::deserialize::{self, FromSql, Queryable};
            use diesel::pg::Pg;
            use diesel::sql_types::BigInt;
            use std::borrow::Borrow;
            use std::fmt::{self, Display};

            #[derive(
                Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialOrd, Ord, PartialEq, Eq,
            )]
            pub struct $name(i64);

            impl $name {
                #[inline]
                pub fn to_i64(self) -> i64 {
                    self.0
                }

                #[inline]
                pub fn from_raw(value: i64) -> Self {
                    debug!("Creating raw {} with value {}", stringify!($name), value);

                    $name(value)
                }
            }

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

            impl Display for $name {
                #[inline]
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write!(f, "{}", self.0)
                }
            }

            impl Queryable<BigInt, Pg> for $name {
                type Row = <i64 as Queryable<BigInt, Pg>>::Row;

                fn build(row: Self::Row) -> Self {
                    let id = <i64 as Queryable<BigInt, Pg>>::build(row);
                    $name(id)
                }
            }

            impl FromSql<BigInt, Pg> for $name {
                #[inline]
                fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
                    let id = <i64 as FromSql<BigInt, Pg>>::from_sql(bytes)?;
                    Ok($name(id))
                }
            }
        }

        pub use $module::$name;
    };
}
