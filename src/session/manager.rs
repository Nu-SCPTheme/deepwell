/*
 * session/manager.rs
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

use super::{NewLoginAttempt, NewSession};
use crate::manager_prelude::*;
use crate::schema::{login_attempts, sessions};
use crate::utils::rows_to_result;
use chrono::prelude::*;
use ipnetwork::IpNetwork;
use rand::{distributions::Alphanumeric, rngs::OsRng, Rng};
use std::iter;

const TOKEN_LENGTH: usize = 64;

make_id_type!(LoginAttemptId);

// This implementation is extremely primitive -- it just stores a securely-generated
// random string as the token and then matches it when the user makes calls.
//
// In the future we will want distinct session objects which are separated by IP and
// can be invalidated separately.
//
// This also might want to be in-memory instead of persisted.

#[derive(Debug, Queryable)]
pub struct Session {
    user_id: UserId,
    token: String,
    ip_address: IpNetwork,
    created_at: DateTime<Utc>,
}

impl Session {
    #[inline]
    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    #[inline]
    pub fn token(&self) -> &str {
        &self.token
    }

    #[inline]
    pub fn ip_address(&self) -> IpNetwork {
        self.ip_address
    }
}

#[derive(Debug, Queryable)]
pub struct LoginAttempt {
    login_attempt_id: LoginAttemptId,
    user_id: UserId,
    ip_address: IpNetwork,
    success: bool,
    attempted_at: DateTime<Utc>,
}

impl LoginAttempt {
    #[inline]
    pub fn login_attempt_id(&self) -> LoginAttemptId {
        self.login_attempt_id
    }

    #[inline]
    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    #[inline]
    pub fn ip_address(&self) -> IpNetwork {
        self.ip_address
    }

    #[inline]
    pub fn success(&self) -> bool {
        self.success
    }

    #[inline]
    pub fn attempted_at(&self) -> DateTime<Utc> {
        self.attempted_at
    }
}

pub struct SessionManager {
    conn: Arc<PgConnection>,
}

impl SessionManager {
    #[inline]
    pub fn new(conn: &Arc<PgConnection>) -> Self {
        let conn = Arc::clone(conn);
        SessionManager { conn }
    }

    pub async fn get_session(&self, user_id: UserId) -> Result<Option<Session>> {
        info!("Getting session information any for user ID {}", user_id);

        let id: i64 = user_id.into();
        let session = sessions::table
            .find(id)
            .first::<Session>(&*self.conn)
            .optional()?;

        Ok(session)
    }

    pub async fn get_token(&self, user_id: UserId) -> Result<Option<String>> {
        debug!("Getting token (if any) for user ID {}", user_id);

        let id: i64 = user_id.into();
        let token = sessions::table
            .find(id)
            .select(sessions::dsl::token)
            .first::<String>(&*self.conn)
            .optional()?;

        Ok(token)
    }

    pub async fn check_token(&self, user_id: UserId, token: &str) -> Result<()> {
        debug!("Checking token for user ID {}", user_id);

        let id: i64 = user_id.into();
        let result = sessions::table
            .find(id)
            .filter(sessions::dsl::token.eq(token))
            .select(sessions::dsl::user_id)
            .first::<UserId>(&*self.conn)
            .optional()?;

        match result {
            Some(_) => Ok(()),
            None => Err(Error::InvalidToken),
        }
    }

    pub async fn create_token(&self, user_id: UserId, ip_address: IpNetwork) -> Result<String> {
        debug!("Creating token for user ID {}", user_id);

        let token = generate_token();
        let model = NewSession {
            user_id: user_id.into(),
            token: &token,
            ip_address,
        };

        diesel::insert_into(sessions::table)
            .values(&model)
            .execute(&*self.conn)?;

        Ok(token)
    }

    pub async fn revoke_token(&self, user_id: UserId) -> Result<bool> {
        debug!("Revoking token for user ID {}", user_id);

        let id: i64 = user_id.into();
        let rows = diesel::delete(sessions::table)
            .filter(sessions::dsl::user_id.eq(id))
            .execute(&*self.conn)?;

        Ok(rows_to_result(rows))
    }

    pub async fn add_login_attempt(
        &self,
        user_id: UserId,
        ip_address: IpNetwork,
        success: bool,
    ) -> Result<LoginAttemptId> {
        debug!(
            "Adding login attempt for user ID {} from {}",
            user_id, ip_address,
        );

        let model = NewLoginAttempt {
            user_id: user_id.into(),
            ip_address,
            success,
        };

        let id = diesel::insert_into(login_attempts::table)
            .values(&model)
            .returning(login_attempts::dsl::login_attempt_id)
            .get_result::<LoginAttemptId>(&*self.conn)?;

        Ok(id)
    }

    pub async fn set_login_success(&self, login_attempt_id: LoginAttemptId) -> Result<()> {
        use login_attempts::dsl;

        debug!(
            "Setting login attempt ID {} as successful",
            login_attempt_id,
        );

        let id: i64 = login_attempt_id.into();
        diesel::update(dsl::login_attempts.filter(dsl::login_attempt_id.eq(id)))
            .set(dsl::success.eq(true))
            .execute(&*self.conn)?;

        Ok(())
    }

    pub async fn get_login_attempts(
        &self,
        user_id: UserId,
        since: DateTime<Utc>,
    ) -> Result<Vec<LoginAttempt>> {
        debug!(
            "Getting login attempts for user ID {} since {}",
            user_id, since,
        );

        let id: i64 = user_id.into();
        let attempts = login_attempts::table
            .filter(login_attempts::attempted_at.gt(since))
            .filter(login_attempts::user_id.eq(id))
            .order_by(login_attempts::attempted_at.desc())
            .get_results::<LoginAttempt>(&*self.conn)?;

        Ok(attempts)
    }
}

impl_async_transaction!(SessionManager);

fn generate_token() -> String {
    iter::repeat(())
        .map(|_| OsRng.sample(Alphanumeric))
        .take(TOKEN_LENGTH)
        .collect()
}
