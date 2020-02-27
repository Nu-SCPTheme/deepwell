/*
 * test/mod.rs
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

extern crate color_backtrace;
extern crate tempfile;
extern crate tokio;

mod authors;
mod factory;
mod lock;
mod login;
mod page;
mod password;
mod session;
mod tags;
mod user;
mod verify;
mod wiki;

mod prelude {
    pub use super::factory::*;
    pub use crate::prelude::*;
    pub use either::*;
}
