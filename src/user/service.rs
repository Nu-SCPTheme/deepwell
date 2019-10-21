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
    cache: (),
}

impl<'d> UserService<'d> {
    #[inline]
    pub fn new(conn: &'d PgConnection) -> Self {
        UserService { conn, cache: () }
    }

    pub fn create(&mut self, name: &str, email: &str) -> Result<()> {
        info!(
            "Creating new user with name '{}' with email '{}'",
            name, email
        );

        let model = NewUser { name, email };
        let user = diesel::insert_into(users::table)
            .values(&model)
            .get_result::<User>(self.conn)?;

        //self.users.insert(user.id(), user);

        Ok(())
    }

    pub fn edit(&mut self, id: UserId, model: UpdateUser) -> Result<()> {
        use self::users::dsl;

        let id: i64 = id.into();
        info!("Editing user id {}, changes: {:?}", id, model);

        diesel::update(dsl::users.filter(dsl::user_id.eq(id)))
            .set(model)
            .execute(self.conn)?;

        Ok(())
    }

    pub fn mark_inactive(&mut self, id: UserId) -> Result<()> {
        use self::users::dsl;
        use diesel::dsl::now;

        let id: i64 = id.into();
        info!("Marking user id {} as inactive", id);

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
