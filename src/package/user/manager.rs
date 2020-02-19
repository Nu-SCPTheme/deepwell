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

use super::models::{NewUser, NewUserVerification, UpdateUser};
use crate::manager_prelude::*;
use crate::schema::{user_verification, users};
use crate::utils::{lower, rand_alphanum, rows_to_result};
use cow_utils::CowUtils;
use diesel::pg::expression::dsl::any;
use ref_map::*;

pub struct UserManager {
    conn: Arc<PgConnection>,
}

impl UserManager {
    #[inline]
    pub fn new(conn: &Arc<PgConnection>) -> Self {
        debug!("Creating user-manager service");

        let conn = Arc::clone(conn);
        UserManager { conn }
    }

    async fn check_conflicts(&self, name: &str, email: &str) -> Result<()> {
        use self::users::dsl;

        let result = users::table
            .filter(lower(users::name).eq(lower(name)))
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

            // If there's a result then one of the email or name conflicts
            unreachable!()
        }

        // No conflicts
        Ok(())
    }

    pub async fn create(&self, name: &str, email: &str) -> Result<UserId> {
        info!(
            "Creating new user with name '{}' with email '{}'",
            name, email,
        );

        let email = email.cow_to_ascii_lowercase();
        self.check_conflicts(name, &email).await?;

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

    pub async fn get_id_from_email_or_name(&self, name_or_email: &str) -> Result<Option<UserId>> {
        info!("Getting user ID for username or email '{}'", name_or_email);

        let result = users::table
            .filter(lower(users::name).eq(lower(name_or_email)))
            .or_filter(users::email.eq(lower(name_or_email)))
            .select(users::dsl::user_id)
            .first::<UserId>(&*self.conn)
            .optional()?;

        Ok(result)
    }

    pub async fn get_from_email(&self, email: &str) -> Result<Option<User>> {
        info!("Getting user for email '{}'", email);

        let result = users::table
            .filter(users::email.eq(lower(email)))
            .first::<User>(&*self.conn)
            .optional()?;

        Ok(result)
    }

    pub async fn get_from_name(&self, name: &str) -> Result<Option<User>> {
        info!("Getting user for name '{}'", name);

        let result = users::table
            .filter(lower(users::name).eq(lower(name)))
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

        let gender = gender.map(|s| s.cow_to_ascii_lowercase());
        let gender = gender.ref_map(|s| s.as_ref());

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

    pub async fn verify_token(&self, token: &str) -> Result<()> {
        debug!("Marking user associated with token '{}' as verified", token);

        self.transaction(async {
            let user_id = user_verification::table
                .filter(user_verification::token.eq(token))
                .select(user_verification::dsl::user_id)
                .first::<UserId>(&*self.conn)
                .optional()?;

            match user_id {
                None => Err(Error::InvalidVerificationToken),
                Some(id) => {
                    self.verify(id).await?;

                    let rows = diesel::delete(user_verification::table)
                        .filter(user_verification::token.eq(token))
                        .execute(&*self.conn)?;

                    if rows_to_result(rows) {
                        Ok(())
                    } else {
                        Err(Error::InvalidVerificationToken)
                    }
                }
            }
        })
        .await
    }

    pub async fn create_token(&self, id: UserId) -> Result<String> {
        info!("Creating new verification token for user ID {}", id);

        let token = rand_alphanum(64);
        let model = NewUserVerification {
            user_id: id.into(),
            token: &token,
        };

        diesel::insert_into(user_verification::table)
            .values(&model)
            .execute(&*self.conn)?;

        Ok(token)
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
