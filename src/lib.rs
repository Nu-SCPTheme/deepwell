/*
 * lib.rs
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

#![deny(missing_debug_implementations)]
#![forbid(unsafe_code)]

extern crate async_std;
extern crate chrono;
extern crate cow_utils;
extern crate crypto;
extern crate deepwell_core;

#[macro_use]
extern crate diesel;
extern crate either;

#[macro_use]
extern crate futures;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;
extern crate map_vec;
extern crate rand;
extern crate ref_map;

#[macro_use]
extern crate serde;
extern crate subprocess;

#[macro_use]
extern crate tinyvec;
extern crate wikidot_normalize;

#[macro_use]
mod macros;

mod package;
mod schema;
mod server;
mod utils;

#[cfg(test)]
mod test;

pub mod prelude {
    pub use crate::package::page::PageCommit;
    pub use crate::server::{Config, Server};
    pub use crate::{Error, Result, StdResult};
    pub use deepwell_core::prelude::*;
}

mod manager_prelude {
    pub use crate::prelude::*;
    pub use crate::schema::*;
    pub use async_std::prelude::*;
    pub use async_std::sync::RwLock;
    pub use chrono::prelude::*;
    pub use diesel::prelude::*;
    pub use diesel::query_builder::debug_query;
    pub use either::{Either, Left, Right};
    pub use std::collections::HashMap;
    pub use std::convert::TryFrom;
    pub use std::fmt::{self, Debug};
    pub use std::sync::Arc;

    // For Option<Option<T>>, updating nullable columns
    pub type Nullable<T> = Option<T>;
}

pub type StdResult<T, E> = std::result::Result<T, E>;
pub type Result<T> = StdResult<T, Error>;

pub use self::server::{Config, Server};
pub use deepwell_core::error::Error;
