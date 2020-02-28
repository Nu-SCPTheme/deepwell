/*
 * server/session.rs
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

use crate::manager_prelude::*;

macro_rules! wrap_login {
    ($future:expr) => {
        match $future.await {
            Ok(session) => Ok(session),
            Err(error) => {
                trace!("Pausing before response due to invalid credentials.");

                password_pause().await;
                Err(error)
            }
        }
    };
}

impl Server {
    /// Attempts to login a user via user ID.
    /// Returns the new session if successful, `AuthenticationFailed` otherwise.
    pub async fn try_login_id(
        &self,
        user_id: UserId,
        password: &str,
        remote_address: Option<&str>,
    ) -> Result<Session> {
        wrap_login!(self.try_login_id_internal(user_id, password, remote_address))
    }

    async fn try_login_id_internal(
        &self,
        user_id: UserId,
        password: &str,
        remote_address: Option<&str>,
    ) -> Result<Session> {
        info!(
            "Trying to login user ID {} (from {})",
            user_id,
            remote_address.unwrap_or("<unknown>"),
        );

        if password.is_empty() {
            // If they filled in the username but not the password,
            // it is possible they accidentally entered the password
            // in the username/email field and hit enter.
            //
            // In such a case we do not want to log their attempt, or
            // we could be recording their password (or something similar)
            // in plaintext.
            //
            // Instead we will bail out with an authentication failure.
            return Err(Error::AuthenticationFailed);
        }

        // Outside of a transaction so it doesn't get rolled back
        let login_attempt_id = self
            .session
            .add_login_attempt(Some(user_id), None, remote_address, false)
            .await?;

        self.transaction(async {
            self.password.check(user_id, password).await?;

            let session = self
                .session
                .create_session(user_id, login_attempt_id)
                .await?;

            Ok(session)
        })
        .await
    }

    /// Attempts to login a user via username or email.
    /// Returns the new session if successful, `AuthenticationFailed` otherwise.
    pub async fn try_login(
        &self,
        name_or_email: &str,
        password: &str,
        remote_address: Option<&str>,
    ) -> Result<Session> {
        wrap_login!(self.try_login_internal(name_or_email, password, remote_address))
    }

    pub async fn try_login_internal(
        &self,
        name_or_email: &str,
        password: &str,
        remote_address: Option<&str>,
    ) -> Result<Session> {
        info!(
            "Trying to login user '{}' (from {})",
            name_or_email,
            remote_address.unwrap_or("<unknown>"),
        );

        // Get associated user, if it exists
        let user_id = self.user.get_id_from_email_or_name(name_or_email).await?;

        // Attempt login or fail
        match user_id {
            Some(id) => self.try_login_id_internal(id, password, remote_address).await,
            None => {
                self.session
                    .add_login_attempt(None, Some(name_or_email), remote_address, false)
                    .await?;

                Err(Error::AuthenticationFailed)
            }
        }
    }

    /// Validate a user's session to ensure they are logged in.
    /// Returns `()` if successful, `NotLoggedIn` otherwise.
    #[inline]
    pub async fn check_session(&self, session_id: SessionId, user_id: UserId) -> Result<()> {
        self.session.check_session(session_id, user_id).await
    }

    /// Deactivate a session currently logged in.
    /// Returns `()` if successful, `NotLoggedIn` if no such session was found.
    #[inline]
    pub async fn end_session(&self, session_id: SessionId, user_id: UserId) -> Result<()> {
        self.session.end_session(session_id, user_id).await
    }

    /// Deactivates all sessions except the one currently logged in.
    /// Returns a list of the sessions which were deactivated.
    pub async fn end_other_sessions(
        &self,
        session_id: SessionId,
        user_id: UserId,
    ) -> Result<Vec<Session>> {
        self.session.end_other_sessions(session_id, user_id).await
    }

    /// Get all sessions for the given user.
    /// Returns the current session (the one passed in the argument) first,
    /// and the other sessions in the list.
    pub async fn get_sessions(
        &self,
        session_id: SessionId,
        user_id: UserId,
    ) -> Result<(Session, Vec<Session>)> {
        self.session.get_sessions(session_id, user_id).await
    }

    /// Fetch login attempt associated with the passed ID.
    #[inline]
    pub async fn get_login_attempt(
        &self,
        login_attempt_id: LoginAttemptId,
    ) -> Result<LoginAttempt> {
        self.session.get_login_attempt(login_attempt_id).await
    }

    /// Returns all login attempts for a user since the given date.
    /// Limited to 100 entries.
    #[inline]
    pub async fn get_login_attempts<Tz: TimeZone>(
        &self,
        user_id: UserId,
        since: DateTime<Tz>,
    ) -> Result<Vec<LoginAttempt>> {
        self.session.get_login_attempts(user_id, since).await
    }

    /// Returns all login attempts for all users since the given date.
    /// Limited to 100 entries.
    #[inline]
    pub async fn get_all_login_attempts<Tz: TimeZone>(
        &self,
        since: DateTime<Tz>,
    ) -> Result<Vec<LoginAttempt>> {
        self.session.get_all_login_attempts(since).await
    }
}

#[cfg(test)]
#[inline]
async fn password_pause() {}

#[cfg(not(test))]
async fn password_pause() {
    use async_std::task;
    use std::time::Duration;

    const PAUSE: Duration = Duration::from_millis(500);

    task::sleep(PAUSE).await;
}
