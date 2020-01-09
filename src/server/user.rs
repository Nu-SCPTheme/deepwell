/*
 * server/user.rs
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
    /// Creates a new user with the given name and email. Returns its ID.
    #[inline]
    pub async fn create_user(&self, name: &str, email: &str, password: &str) -> Result<UserId> {
        self.transaction(async {
            let user_id = self.user.create(name, email).await?;
            self.password.set(user_id, password).await?;

            Ok(user_id)
        })
        .await
    }

    /// Edits data attached to a user with the given ID.
    #[inline]
    pub async fn edit_user(&self, id: UserId, changes: UserMetadata<'_>) -> Result<()> {
        self.user.edit(id, changes).await
    }

    /// Get the model for a user from its ID.
    #[inline]
    pub async fn get_user_from_id(&self, id: UserId) -> Result<User> {
        self.user.get_from_id(id).await?.ok_or(Error::UserNotFound)
    }

    /// Gets the models for users from their IDs.
    /// Results are returned in the same order as the IDs, and any missing
    /// users give `None` instead.
    #[inline]
    pub async fn get_users_from_ids(&self, ids: &[UserId]) -> Result<Vec<Option<User>>> {
        self.user.get_from_ids(ids).await
    }

    /// Gets the model for a user from its name.
    #[inline]
    pub async fn get_user_from_name(&self, name: &str) -> Result<Option<User>> {
        self.user.get_from_name(name).await
    }

    /// Gets the model for a user from its email.
    #[inline]
    pub async fn get_user_from_email(&self, email: &str) -> Result<Option<User>> {
        self.user.get_from_email(email).await
    }

    /// Marks a user as verified.
    #[inline]
    pub async fn verify_user(&self, id: UserId) -> Result<()> {
        self.user.verify(id).await
    }

    /// Marks the user as "inactive", effectively deleting them.
    #[inline]
    pub async fn mark_user_inactive(&self, id: UserId) -> Result<()> {
        self.user.mark_inactive(id, true).await
    }

    /// Marks the user as "active" again, effectively un-deleting them.
    #[inline]
    pub async fn mark_user_active(&self, id: UserId) -> Result<()> {
        self.user.mark_inactive(id, false).await
    }
}
