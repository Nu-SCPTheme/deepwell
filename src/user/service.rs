/*
 * user/service.rs
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

use super::models::{NewUser, UpdateUser};
use super::object::{User, UserId};
use crate::schema::users;
use crate::service_prelude::*;

pub struct UserService<'d> {
    conn: &'d PgConnection,
}

impl<'d> UserService<'d> {
    #[inline]
    pub fn new(conn: &'d PgConnection) -> Self {
        UserService { conn }
    }

    pub fn create(&self, name: &str, email: &str) -> Result<()> {
        info!(
            "Creating new user with name '{}' with email '{}'",
            name, email
        );

        let model = NewUser { name, email };
        diesel::insert_into(users::table)
            .values(&model)
            .execute(self.conn)?;

        Ok(())
    }

    pub fn get(&self, id: UserId) -> Result<Option<User>> {
        info!("Getting user for id {}", id);

        let id: i64 = id.into();
        let result = users::table.find(id).first::<User>(self.conn).optional()?;
        Ok(result)
    }

    pub fn edit(&self, id: UserId, model: UpdateUser) -> Result<()> {
        use self::users::dsl;

        info!("Editing user id {}, changes: {:?}", id, model);

        let id: i64 = id.into();
        diesel::update(dsl::users.filter(dsl::user_id.eq(id)))
            .set(model)
            .execute(self.conn)?;

        Ok(())
    }

    pub fn mark_inactive(&self, id: UserId) -> Result<()> {
        use self::users::dsl;
        use diesel::dsl::now;

        info!("Marking user id {} as inactive", id);

        let id: i64 = id.into();
        diesel::update(dsl::users.filter(dsl::user_id.eq(id)))
            .set(dsl::deleted_at.eq(now))
            .execute(self.conn)?;

        Ok(())
    }
}

impl Debug for UserService<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("UserService")
            .field("conn", &"PgConnection { .. }")
            .finish()
    }
}
