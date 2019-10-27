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
    pub fn is_active(&self) -> bool {
        self.deleted_at.is_none()
    }
}

pub struct UserService {
    conn: Arc<PgConnection>,
}

impl UserService {
    #[inline]
    pub fn new(conn: &Arc<PgConnection>) -> Self {
        let conn = Arc::clone(conn);
        UserService { conn }
    }

    pub fn create(&self, name: &str, email: &str) -> Result<UserId> {
        use self::users::dsl;

        info!(
            "Starting transaction to create new user with name '{}' with email '{}'",
            name, email,
        );

        let email = email.to_ascii_lowercase();

        self.conn.transaction::<_, Error, _>(|| {
            let result = users::table
                .filter(users::name.eq(name))
                .or_filter(users::email.eq(&email))
                .get_result::<User>(&*self.conn)
                .optional()?;

            if let Some(user) = result {
                if name == &user.name {
                    warn!("Cannot create user, name conflicts");
                    return Err(Error::UserNameExists);
                }

                if email == user.email {
                    warn!("Cannot create user, email conflicts");
                    return Err(Error::UserEmailExists);
                }

                unreachable!()
            }

            let model = NewUser {
                name,
                email: &email,
            };

            let id = diesel::insert_into(users::table)
                .values(&model)
                .returning(users::dsl::user_id)
                .get_result::<UserId>(&*self.conn)?;

            Ok(id)
        })
    }

    pub fn get(&self, id: UserId) -> Result<Option<User>> {
        info!("Getting user for id {}", id);

        let id: i64 = id.into();
        let result = users::table
            .find(id)
            .first::<User>(&*self.conn)
            .optional()?;
        Ok(result)
    }

    pub fn edit(
        &self,
        id: UserId,
        name: Option<&str>,
        email: Option<&str>,
        author_page: Option<&str>,
        website: Option<&str>,
        about: Option<&str>,
        gender: Option<&str>,
        location: Option<&str>,
    ) -> Result<()> {
        use self::users::dsl;

        let is_verified = if email.is_some() { Some(false) } else { None };
        let model = UpdateUser {
            name,
            email,
            is_verified,
            author_page,
            website,
            about,
            gender,
            location,
            deleted_at: None,
        };

        info!("Editing user id {}, data: {:?}", id, &model);

        let id: i64 = id.into();
        diesel::update(dsl::users.filter(dsl::user_id.eq(id)))
            .set(&model)
            .execute(&*self.conn)?;

        Ok(())
    }

    pub fn verify(&self, id: UserId) -> Result<()> {
        use self::users::dsl;

        info!("Marking user id {} as verified", id);

        let id: i64 = id.into();
        diesel::update(dsl::users.filter(dsl::user_id.eq(id)))
            .set(dsl::is_verified.eq(true))
            .execute(&*self.conn)?;

        Ok(())
    }

    pub fn mark_inactive(&self, id: UserId, value: bool) -> Result<()> {
        use self::users::dsl;
        use diesel::dsl::now;

        info!(
            "Marking user id {} as {}",
            id,
            if value { "inactive" } else { "active" }
        );

        let id: i64 = id.into();
        let condition = dsl::users.filter(dsl::user_id.eq(id));

        // Set to NOW() or NULL
        if value {
            diesel::update(condition)
                .set(dsl::deleted_at.eq(now))
                .execute(&*self.conn)?;
        } else {
            let model = UpdateUser {
                name: None,
                email: None,
                is_verified: None,
                author_page: None,
                website: None,
                about: None,
                gender: None,
                location: None,
                deleted_at: Some(None),
            };

            diesel::update(condition).set(&model).execute(&*self.conn)?;
        }

        Ok(())
    }
}

impl Debug for UserService {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("UserService")
            .field("conn", &"PgConnection { .. }")
            .finish()
    }
}
