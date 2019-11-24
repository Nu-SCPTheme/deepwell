/*
 * error.rs
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

use diesel::result::{ConnectionError, Error as DieselError};
use std::io;
use subprocess::PopenError;

#[derive(Debug, Error)]
pub enum Error {
    #[error("uncommon error: {0}")]
    StaticMsg(&'static str),

    #[error("general I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("database error: {0}")]
    Database(#[from] DieselError),

    #[error("error connecting to database: {0}")]
    DatabaseConnection(#[from] ConnectionError),

    #[error("error running subprocess: {0}")]
    Subprocess(#[from] PopenError),

    #[error("command failed: {0}")]
    CommandFailed(String),

    #[error("invalid username or password")]
    AuthenticationFailed,

    #[error("session expired or invalid")]
    InvalidToken,

    #[error("invalid password: {0}")]
    NewPasswordInvalid(&'static str),

    #[error("the given wiki was not found")]
    WikiNotFound,

    #[error("the given page was not found")]
    PageNotFound,

    #[error("the given page already exists")]
    PageExists,

    #[error("the given user was not found")]
    UserNotFound,

    #[error("a user with the given name already exists")]
    UserNameExists,

    #[error("a user with the given email already exists")]
    UserEmailExists,

    #[error("the given revision was not found")]
    RevisionNotFound,

    #[error("the given revision does not correspond to the specified page")]
    RevisionPageMismatch,
}
