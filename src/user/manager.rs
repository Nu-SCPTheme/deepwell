/*
 * user/manager.rs
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

use super::models::{NewUser, UpdateUser};
use crate::manager_prelude::*;
use crate::schema::users;
use diesel::pg::expression::dsl::any;

#[derive(Serialize, Deserialize, Queryable, Debug, Clone, PartialEq, Eq)]
pub struct User {
    user_id: UserId,
    name: String,
    email: String,
    is_verified: bool,
    is_bot: bool,
    author_page: String,
    website: String,
    about: String,
    gender: String,
    location: String,
    created_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
}

impl User {
    #[inline]
    pub fn id(&self) -> UserId {
        self.user_id
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
    pub fn is_bot(&self) -> bool {
        self.is_bot
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
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    #[inline]
    pub fn deleted_at(&self) -> Option<DateTime<Utc>> {
        self.deleted_at
    }

    #[inline]
    pub fn is_active(&self) -> bool {
        self.deleted_at.is_none()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct UserMetadata<'a> {
    pub name: Option<&'a str>,
    pub email: Option<&'a str>,
    pub author_page: Option<&'a str>,
    pub website: Option<&'a str>,
    pub about: Option<&'a str>,
    pub gender: Option<&'a str>,
    pub location: Option<&'a str>,
}

pub struct UserManager {
    conn: Arc<PgConnection>,
}

impl UserManager {
    #[inline]
    pub fn new(conn: &Arc<PgConnection>) -> Self {
        let conn = Arc::clone(conn);
        UserManager { conn }
    }

    pub async fn create(&self, name: &str, email: &str) -> Result<UserId> {
        use self::users::dsl;

        info!(
            "Starting transaction to create new user with name '{}' with email '{}'",
            name, email,
        );

        let email = email.to_ascii_lowercase();

        self.transaction(async {
            // Check if there any existing users
            let result = users::table
                .filter(users::name.eq(name))
                .or_filter(users::email.eq(&email))
                .select((dsl::user_id, dsl::name, dsl::email))
                .get_result::<(UserId, String, String)>(&*self.conn)
                .optional()?;

            if let Some((user_id, conflict_name, conflict_email)) = result {
                if name == conflict_name {
                    warn!("Cannot create user, name conflicts with ID {}", user_id);
                    return Err(Error::UserNameExists);
                }

                if email == conflict_email {
                    warn!("Cannot create user, email conflicts with ID {}", user_id);
                    return Err(Error::UserEmailExists);
                }

                unreachable!()
            }

            // If not, insert into database
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
        .await
    }

    pub async fn get_from_id(&self, id: UserId) -> Result<Option<User>> {
        info!("Getting users for id: {}", id);

        let id: i64 = id.into();
        let result = users::table
            .filter(users::user_id.eq(id))
            .first::<User>(&*self.conn)
            .optional()?;

        Ok(result)
    }

    pub async fn get_from_ids(&self, ids: &[UserId]) -> Result<Vec<Option<User>>> {
        info!("Getting users for ids: {:?}", ids);

        // Load
        let mut result = {
            let ids: Vec<_> = ids.iter().map(|id| id.to_i64()).collect();
            users::table
                .filter(users::user_id.eq(any(ids)))
                .order_by(users::user_id.asc())
                .load::<User>(&*self.conn)?
        };

        // Add in nones where needed
        let mut users = Vec::new();
        for id in ids.iter().copied() {
            // Use Vec.remove() to get an owned version for the final Vec
            let user = result
                .iter()
                .enumerate()
                .map(|(idx, user)| (idx, user.id()))
                .find(|(_, user_id)| *user_id == id)
                .map(|(idx, _)| result.remove(idx));

            users.push(user);
        }

        debug_assert!(result.is_empty());

        Ok(users)
    }

    pub async fn get_from_name(&self, name: &str) -> Result<Option<User>> {
        info!("Getting user for name '{}'", name);

        let result = users::table
            .filter(users::name.eq(name))
            .first::<User>(&*self.conn)
            .optional()?;

        Ok(result)
    }

    pub async fn get_from_email(&self, email: &str) -> Result<Option<User>> {
        info!("Getting user for email '{}'", email);

        let result = users::table
            .filter(users::email.eq(email))
            .first::<User>(&*self.conn)
            .optional()?;

        Ok(result)
    }

    pub async fn edit(&self, id: UserId, changes: UserMetadata<'_>) -> Result<()> {
        use self::users::dsl;

        // Extract fields from metadata struct
        let UserMetadata {
            name,
            email,
            author_page,
            website,
            about,
            gender,
            location,
        } = changes;

        let gender = gender.map(|s| s.to_ascii_lowercase());
        // Allocate for lowercase'd version, then take reference
        let gender = gender.as_ref().map(|s| s.as_str());

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

        info!("Editing user ID {}, data: {:?}", id, &model);

        let id: i64 = id.into();
        diesel::update(dsl::users.filter(dsl::user_id.eq(id)))
            .set(&model)
            .execute(&*self.conn)?;

        Ok(())
    }

    pub async fn verify(&self, id: UserId) -> Result<()> {
        use self::users::dsl;

        info!("Marking user ID {} as verified", id);

        let id: i64 = id.into();
        diesel::update(dsl::users.filter(dsl::user_id.eq(id)))
            .set(dsl::is_verified.eq(true))
            .execute(&*self.conn)?;

        Ok(())
    }

    pub async fn mark_inactive(&self, id: UserId, value: bool) -> Result<()> {
        use self::users::dsl;
        use diesel::dsl::now;

        info!(
            "Marking user ID {} as {}",
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

impl_async_transaction!(UserManager);

impl Debug for UserManager {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("UserManager")
            .field("conn", &"PgConnection { .. }")
            .finish()
    }
}
