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
use crate::schema::users;
use crate::service_prelude::*;

make_id_type!(UserId);

#[derive(Serialize, Deserialize, Queryable, Debug, Clone, PartialEq, Eq)]
pub struct User {
    id: UserId,
    name: String,
    email: String,
    is_verified: bool,
    author_page: String,
    website: String,
    about: String,
    gender: String,
    location: String,
    created_at: NaiveDateTime,
    deleted_at: Option<NaiveDateTime>,
}

impl User {
    #[inline]
    pub fn id(&self) -> UserId {
        self.id
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn email(&self) -> &str {
        &self.email
    }

    #[inline]
    pub fn is_verified(&self) -> bool {
        self.is_verified
    }

    #[inline]
    pub fn author_page(&self) -> &str {
        &self.author_page
    }

    #[inline]
    pub fn website(&self) -> &str {
        &self.website
    }

    #[inline]
    pub fn about(&self) -> &str {
        &self.about
    }

    #[inline]
    pub fn gender(&self) -> &str {
        &self.gender
    }

    #[inline]
    pub fn location(&self) -> &str {
        &self.location
    }

    #[inline]
    pub fn created_at(&self) -> NaiveDateTime {
        self.created_at
    }

    #[inline]
    pub fn deleted_at(&self) -> Option<NaiveDateTime> {
        self.deleted_at
    }

    #[inline]
    pub fn is_active(&self) -> bool {
        self.deleted_at.is_none()
    }
}

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
