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

impl Server {
    /// Attempts to login a user via user ID, returning `()` if successful.
    pub async fn try_login_id(
        &self,
        user_id: UserId,
        password: &str,
        ip_address: IpNetwork,
    ) -> Result<()> {
        info!("Trying to login user ID {} (from {})", user_id, ip_address);

        // Outside of a transaction so it doesn't get rolled back
        let login_attempt_id = self
            .session
            .add_login_attempt(Some(user_id), None, ip_address, false)
            .await?;

        self.password.check(user_id, password).await?;
        self.session.set_login_success(login_attempt_id).await?;

        Ok(())
    }

    /// Attempts to login a user via username or email, returning `()` if successful.
    pub async fn try_login(
        &self,
        name_or_email: &str,
        password: &str,
        ip_address: IpNetwork,
    ) -> Result<()> {
        info!(
            "Trying to login user '{}' (from {})",
            name_or_email, ip_address,
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

        // Get associated user, if it exists
        let user_id = self.user.get_id_from_email_or_name(name_or_email).await?;

        // Attempt login or fail
        match user_id {
            Some(id) => self.try_login_id(id, password, ip_address).await,
            None => {
                self.session
                    .add_login_attempt(None, Some(name_or_email), ip_address, false)
                    .await?;

                Err(Error::AuthenticationFailed)
            }
        }
    }

    /// Returns all login attempts for a user since the given date.
    #[inline]
    pub async fn get_login_attempts(
        &self,
        user_id: UserId,
        since: DateTime<Utc>,
    ) -> Result<Vec<LoginAttempt>> {
        self.transaction(async {
            let user = self
                .user
                .get_from_id(user_id)
                .await?
                .ok_or(Error::UserNotFound)?;

            self.session.get_login_attempts(&user, since).await
        })
        .await
    }
}
