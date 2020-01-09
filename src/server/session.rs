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
use crate::session::Session;

impl Server {
    /// Checks if a given token is valid for the given user.
    #[inline]
    pub async fn check_token(&self, user_id: UserId, token: &str) -> Result<()> {
        self.session.check_token(user_id, token).await
    }

    /// Creates a session by validating the password and creating a token.
    pub async fn create_session(
        &self,
        user_id: UserId,
        password: &str,
        ip_address: IpNetwork,
    ) -> Result<String> {
        info!(
            "Trying to create session for user ID {} (from {})",
            user_id, ip_address,
        );

        // Is outside of the transaction so it doesn't get rolled back on failure
        let login_attempt_id = self
            .session
            .add_login_attempt(user_id, ip_address, false)
            .await?;

        trace!("Password validated, getting or creating session token");
        self.transaction(async {
            self.password.check(user_id, password).await?;
            self.session.set_login_success(login_attempt_id).await?;

            let result = self.session.get_token(user_id).await?;
            if let Some(token) = result {
                return Ok(token);
            }

            self.session.create_token(user_id, ip_address).await
        })
        .await
    }

    /// Gets an existing session object for a given user.
    #[inline]
    pub async fn get_session(&self, user_id: UserId) -> Result<Option<Session>> {
        self.session.get_session(user_id).await
    }

    /// Invalidates a session object manually.
    /// Returns true if there was a session present.
    #[inline]
    pub async fn end_session(&self, user_id: UserId) -> Result<bool> {
        self.session.revoke_token(user_id).await
    }

    /// Returns all login attempts for a user since the given date.
    #[inline]
    pub async fn get_login_attempts(
        &self,
        user_id: UserId,
        since: DateTime<Utc>,
    ) -> Result<Vec<LoginAttempt>> {
        self.session.get_login_attempts(user_id, since).await
    }
}
