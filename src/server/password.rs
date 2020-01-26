/*
 * server/password.rs
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
use async_std::task;

impl Server {
    /// Sets or overwrites the given user's password.
    #[inline]
    pub fn set_user_password(&self, user_id: UserId, password: &str) -> Result<()> {
        if password.is_empty() {
            return Err(Error::NewPasswordInvalid("passwords may not be empty"));
        }

        task::block_on(self.password.set(user_id, password))?;
        Ok(())
    }

    /// Validates the password for the given user.
    /// Returns `()` on success, authentication error on failure.
    #[inline]
    pub fn validate_user_password(&self, user_id: UserId, password: &str) -> Result<()> {
        task::block_on(self.password.check(user_id, password))?;
        Ok(())
    }
}
