/*
 * error.rs
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

use crate::roles::Role;
use crate::types::UserId;
use diesel::result::{ConnectionError, Error as DieselError};
use std::fmt::{self, Display};
use std::io;
use std::string::FromUtf8Error;
use subprocess::PopenError;

pub type StdResult<T, E> = std::result::Result<T, E>;
pub type Result<T> = StdResult<T, Error>;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("error: {0}")]
    StaticMsg(&'static str),

    #[error("general I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("bytes were not valid UTF-8: {0}")]
    Utf8(#[from] FromUtf8Error),

    #[error("database error: {0}")]
    Database(#[from] DieselError),

    #[error("error connecting to database: {0}")]
    DatabaseConnection(#[from] ConnectionError),

    #[error("error running subprocess: {0}")]
    Subprocess(#[from] PopenError),

    #[error("command failed: {0}")]
    CommandFailed(String),

    #[error("unable to communicate with service: {0}")]
    ServiceTransport(io::Error),

    #[error("request was too large, {0} > {1}")]
    RequestTooLarge(usize, usize),

    #[error("invalid username or password")]
    AuthenticationFailed,

    #[error("not logged in, expired session, or invalid token")]
    InvalidSession,

    #[error("invalid password: {0}")]
    NewPasswordInvalid(&'static str),

    #[error("invalid verification token")]
    InvalidVerificationToken,

    #[error("insufficient permissions, can only be done at {1} or higher, not {0}")]
    InsufficientPermissions(Role, Role),

    #[error("the given wiki was not found")]
    WikiNotFound,

    #[error("the given page was not found")]
    PageNotFound,

    #[error("the given page already exists")]
    PageExists,

    #[error("the page cannot be edited because a lock is present")]
    PageLocked(UserId),

    #[error("a page lock for the given user does not exist")]
    PageLockNotFound,

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

impl Error {
    pub fn fixed_name(&self) -> &'static str {
        use self::Error::*;

        match *self {
            StaticMsg(_) => "custom",
            Io(_) => "io",
            Utf8(_) => "utf-8",
            Database(_) => "database",
            DatabaseConnection(_) => "database-connection",
            Subprocess(_) => "subprocess",
            CommandFailed(_) => "command-failed",
            ServiceTransport(_) => "service-transport",
            RequestTooLarge(_, _) => "request-too-large",
            AuthenticationFailed => "authentication-failed",
            InvalidSession => "invalid-session",
            NewPasswordInvalid(_) => "invalid-password",
            InvalidVerificationToken => "invalid-verification-token",
            InsufficientPermissions(_, _) => "insufficient-permissions",
            WikiNotFound => "wiki-not-found",
            PageNotFound => "page-not-found",
            PageExists => "page-exists",
            PageLocked(_) => "page-locked",
            PageLockNotFound => "page-lock-not-found",
            UserNotFound => "user-not-found",
            UserNameExists => "user-name-exists",
            UserEmailExists => "user-email-exists",
            RevisionNotFound => "revision-not-found",
            RevisionPageMismatch => "revision-page-mismatch",
        }
    }

    #[inline]
    pub fn to_sendable(&self) -> SendableError {
        SendableError {
            name: self.fixed_name().into(),
            message: self.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SendableError {
    name: String,
    message: String,
}

impl SendableError {
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl Display for SendableError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} [{}]", self.message, self.name)
    }
}

impl Into<(String, String)> for SendableError {
    #[inline]
    fn into(self) -> (String, String) {
        let Self { name, message } = self;

        (name, message)
    }
}
