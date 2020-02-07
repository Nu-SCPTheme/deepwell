/*
 * models/login_attempt.rs
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

use super::prelude::*;

#[derive(Debug, Queryable)]
pub struct LoginAttempt {
    id: LoginAttemptId,
    user_id: Option<UserId>,
    username_or_email: Option<String>,
    remote_address: Option<String>,
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
    pub fn remote_address(&self) -> Option<&str> {
        self.remote_address.ref_map(|s| s.as_str())
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
