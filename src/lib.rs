/*
 * lib.rs
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

#![deny(missing_debug_implementations)]

extern crate arrayvec;
extern crate chrono;

#[macro_use]
extern crate diesel;
extern crate hex;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;
extern crate parking_lot;
extern crate regex;

#[macro_use]
extern crate serde;
extern crate serde_json;
extern crate subprocess;

#[macro_use]
extern crate thiserror;
extern crate wikidot_normalize;

mod error;
mod pages;
mod revisions;
mod schema;
mod wikis;

mod service_prelude {
    pub use crate::schema::*;
    pub use crate::Result;
    pub use chrono::prelude::*;
    pub use diesel::prelude::*;
    pub use std::collections::HashMap;
    pub use std::fmt::{self, Debug};
}

pub type StdResult<T, E> = std::result::Result<T, E>;
pub type Result<T> = StdResult<T, Error>;

pub use self::error::Error;
