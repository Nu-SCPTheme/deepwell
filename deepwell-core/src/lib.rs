/*
 * lib.rs
 *
 * deepwell-core - Database management and migrations service
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

#![forbid(unsafe_code)]

extern crate chrono;

#[macro_use]
extern crate diesel;
extern crate subprocess;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;
extern crate map_vec;
extern crate ref_map;
extern crate regex;

#[macro_use]
extern crate serde;

#[macro_use]
extern crate thiserror;

#[macro_use]
mod macros;

pub mod error;
pub mod models;
pub mod types;

pub mod prelude {
    pub use super::error::{Error, SendableError};
    pub use super::models::*;
    pub use super::types::*;
}

pub use self::prelude::*;

pub type StdResult<T, E> = std::result::Result<T, E>;
pub type Result<T> = StdResult<T, Error>;
