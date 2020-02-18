/*
 * models/mod.rs
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

mod blame;
mod git_hash;
mod login_attempt;
mod page;
mod rating;
mod session;
mod user;
mod wiki;

mod prelude {
    pub use crate::error::*;
    pub use crate::types::*;
    pub use chrono::prelude::*;
    pub use ref_map::*;
}

pub use self::blame::Blame;
pub use self::git_hash::GitHash;
pub use self::login_attempt::LoginAttempt;
pub use self::page::Page;
pub use self::rating::Rating;
pub use self::session::Session;
pub use self::user::{User, UserMetadata, UserMetadataOwned};
pub use self::wiki::{Wiki, WikiSettings};
