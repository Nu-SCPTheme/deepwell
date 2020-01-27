/*
 * session/manager.rs
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

use super::NewLoginAttempt;
use crate::manager_prelude::*;
use crate::schema::login_attempts;
use chrono::prelude::*;
use ipnetwork::IpNetwork;
use ref_map::*;

#[derive(Debug, Queryable)]
pub struct LoginAttempt {
    id: LoginAttemptId,
    user_id: Option<UserId>,
    username_or_email: Option<String>,
    ip_address: IpNetwork,
    success: bool,
    attempted_at: DateTime<Utc>,
}

impl LoginAttempt {
    #[inline]
    pub fn login_attempt_id(&self) -> LoginAttemptId {
        self.id
    }

    #[inline]
    pub fn user_id(&self) -> Option<UserId> {
        self.user_id
    }

    #[inline]
    pub fn username_or_email(&self) -> Option<&str> {
        self.username_or_email.ref_map(|s| s.as_str())
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

    pub async fn add_login_attempt(
        &self,
        user_id: Option<UserId>,
        username_or_email: Option<&str>,
        ip_address: IpNetwork,
        success: bool,
    ) -> Result<LoginAttemptId> {
        debug!(
            "Adding login attempt for user ID {:?} / name {:?} from {}",
            user_id, username_or_email, ip_address,
        );

        let model = NewLoginAttempt {
            user_id: user_id.map(|id| id.into()),
            username_or_email,
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
            .limit(100)
            .get_results::<LoginAttempt>(&*self.conn)?;

        Ok(attempts)
    }

    pub async fn get_all_login_attempts(&self, since: DateTime<Utc>) -> Result<Vec<LoginAttempt>> {
        debug!("Getting all login attempts for since {}", since);

        let attempts = login_attempts::table
            .filter(login_attempts::attempted_at.gt(since))
            .order_by(login_attempts::attempted_at.desc())
            .limit(100)
            .get_results::<LoginAttempt>(&*self.conn)?;

        Ok(attempts)
    }
}

impl_async_transaction!(SessionManager);
