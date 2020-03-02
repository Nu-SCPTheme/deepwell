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

use super::{NewLoginAttempt, NewSession};
use crate::manager_prelude::*;
use crate::schema::{login_attempts, sessions};
use crate::utils::rows_to_result;
use chrono::prelude::*;

pub struct SessionManager {
    conn: Arc<PgConnection>,
}

impl SessionManager {
    #[inline]
    pub fn new(conn: &Arc<PgConnection>) -> Self {
        debug!("Creating session-manager service");

        let conn = Arc::clone(conn);
        SessionManager { conn }
    }

    pub async fn add_login_attempt(
        &self,
        user_id: Option<UserId>,
        username_or_email: Option<&str>,
        remote_address: Option<&str>,
        success: bool,
    ) -> Result<LoginAttemptId> {
        {
            // Logging call
            let remote_address = remote_address.unwrap_or("<unknown>");

            match user_id {
                Some(id) => {
                    debug!(
                        "Adding login attempt for user ID {} from {}",
                        id, remote_address,
                    );
                }
                None => {
                    let name = username_or_email
                        .expect("One of user_id or username_or_email must be Some(_)");

                    debug!(
                        "Adding login attempt for user '{}' from {}",
                        name, remote_address,
                    );
                }
            }
        }

        let model = NewLoginAttempt {
            user_id: user_id.map(|id| id.into()),
            username_or_email,
            remote_address,
            success,
        };

        let id = diesel::insert_into(login_attempts::table)
            .values(&model)
            .returning(login_attempts::dsl::login_attempt_id)
            .get_result::<LoginAttemptId>(&*self.conn)?;

        Ok(id)
    }

    pub async fn create_session(
        &self,
        user_id: UserId,
        login_attempt_id: LoginAttemptId,
    ) -> Result<Session> {
        use login_attempts::dsl;

        debug!(
            "Creating a session for user ID {} after login attempt ID {}",
            user_id, login_attempt_id,
        );

        let user_id = user_id.into();
        let login_attempt_id = login_attempt_id.into();

        // Mark login attempt as successful
        diesel::update(dsl::login_attempts.filter(dsl::login_attempt_id.eq(login_attempt_id)))
            .set(dsl::success.eq(true))
            .execute(&*self.conn)?;

        // Add session
        let model = NewSession {
            user_id,
            login_attempt_id,
        };

        let session = diesel::insert_into(sessions::table)
            .values(&model)
            .returning((
                sessions::dsl::session_id,
                sessions::dsl::user_id,
                sessions::dsl::login_attempt_id,
            ))
            .get_result::<Session>(&*self.conn)?;

        Ok(session)
    }

    pub async fn check_session(&self, session_id: SessionId, user_id: UserId) -> Result<()> {
        debug!("Checking session ID {} for user ID {}", session_id, user_id);

        let session: i64 = session_id.into();
        let user: i64 = user_id.into();
        let result = sessions::table
            .filter(sessions::session_id.eq(session))
            .filter(sessions::user_id.eq(user))
            .first::<Session>(&*self.conn)
            .optional()?;

        match result {
            Some(_) => Ok(()),
            None => Err(Error::InvalidSession),
        }
    }

    pub async fn end_session(&self, session_id: SessionId, user_id: UserId) -> Result<()> {
        debug!("Ending session ID {} for user ID {}", session_id, user_id);

        let session: i64 = session_id.into();
        let user: i64 = user_id.into();
        let rows = diesel::delete(sessions::table)
            .filter(sessions::session_id.eq(session))
            .filter(sessions::user_id.eq(user))
            .execute(&*self.conn)?;

        if rows_to_result(rows) {
            Ok(())
        } else {
            Err(Error::InvalidSession)
        }
    }

    pub async fn end_other_sessions(
        &self,
        session_id: SessionId,
        user_id: UserId,
    ) -> Result<Vec<Session>> {
        debug!(
            "Ending all other sessions except ID {} for user ID {}",
            session_id, user_id,
        );

        self.transaction(async {
            // Get sessions to invalidate
            let (_, others) = self.get_sessions(session_id, user_id).await?;

            let user: i64 = user_id.into();
            let other_ids = others
                .iter()
                .map(|session| -> i64 { session.session_id().into() });

            // Remove from active sessions table
            diesel::delete(sessions::table)
                .filter(sessions::session_id.eq_any(other_ids))
                .filter(sessions::user_id.eq(user))
                .execute(&*self.conn)?;

            Ok(others)
        })
        .await
    }

    pub async fn get_sessions(
        &self,
        session_id: SessionId,
        user_id: UserId,
    ) -> Result<(Session, Vec<Session>)> {
        debug!(
            "Getting all sessions for user ID {} (current session ID {})",
            user_id, session_id,
        );

        // Get all sessions for a user
        let id: i64 = user_id.into();
        let mut sessions = sessions::table
            .filter(sessions::user_id.eq(id))
            .get_results::<Session>(&*self.conn)?;

        // Pick out the current session
        let mut current = None;
        for (idx, session) in sessions.iter().enumerate() {
            if session.session_id() == session_id {
                current = Some(idx);
                break;
            }
        }

        // Return an error if there is no current session
        match current {
            None => Err(Error::InvalidSession),
            Some(idx) => {
                let current = sessions.remove(idx);

                Ok((current, sessions))
            }
        }
    }

    pub async fn get_login_attempt(
        &self,
        login_attempt_id: LoginAttemptId,
    ) -> Result<LoginAttempt> {
        debug!("Getting login attempt with ID {}", login_attempt_id);

        let id: i64 = login_attempt_id.into();
        let attempt = login_attempts::table
            .find(id)
            .first::<LoginAttempt>(&*self.conn)?;

        Ok(attempt)
    }

    pub async fn get_login_attempts<Tz: TimeZone>(
        &self,
        user_id: UserId,
        since: DateTime<Tz>,
    ) -> Result<Vec<LoginAttempt>> {
        debug!(
            "Getting login attempts for user ID {} since {}",
            user_id,
            since.time(),
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

    pub async fn get_all_login_attempts<Tz: TimeZone>(
        &self,
        since: DateTime<Tz>,
    ) -> Result<Vec<LoginAttempt>> {
        debug!("Getting all login attempts for since {}", since.time());

        let attempts = login_attempts::table
            .filter(login_attempts::attempted_at.gt(since))
            .order_by(login_attempts::attempted_at.desc())
            .limit(100)
            .get_results::<LoginAttempt>(&*self.conn)?;

        Ok(attempts)
    }
}

impl_async_transaction!(SessionManager);
