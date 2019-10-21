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
use lru_time_cache::LruCache;
use std::sync::Arc;

const DEFAULT_CACHE_SIZE: usize = 4096;

pub struct UserService<'d> {
    conn: &'d PgConnection,
    cache: LruCache<UserId, Arc<User>>,
}

impl<'d> UserService<'d> {
    #[inline]
    pub fn new(conn: &'d PgConnection, cache_size: Option<usize>) -> Self {
        let cache = LruCache::with_capacity(cache_size.unwrap_or(DEFAULT_CACHE_SIZE));

        UserService { conn, cache }
    }

    pub fn create(&mut self, name: &str, email: &str) -> Result<Arc<User>> {
        info!(
            "Creating new user with name '{}' with email '{}'",
            name, email
        );

        let model = NewUser { name, email };
        let user = diesel::insert_into(users::table)
            .values(&model)
            .get_result::<User>(self.conn)?;

        let user = Arc::new(user);
        self.cache.insert(user.id(), Arc::clone(&user));
        Ok(user)
    }

    pub fn edit(&mut self, id: UserId, model: UpdateUser) -> Result<Arc<User>> {
        use self::users::dsl;

        info!("Editing user id {}, changes: {:?}", id, model);

        let id: i64 = id.into();
        let user = diesel::update(dsl::users.filter(dsl::user_id.eq(id)))
            .set(model)
            .get_result::<User>(self.conn)?;

        let user = Arc::new(user);
        self.cache.insert(user.id(), Arc::clone(&user));
        Ok(user)
    }

    pub fn get(&mut self, id: UserId) -> Result<Option<Arc<User>>> {
        info!("Getting user for id {}", id);

        if let Some(user) = self.cache.get(&id) {
            debug!("Found user in cache");
            return Ok(Some(Arc::clone(user)));
        }

        let id: i64 = id.into();
        let result = users::table.find(id).first::<User>(self.conn).optional()?;
        match result {
            Some(user) => {
                let user = Arc::new(user);
                self.cache.insert(user.id(), Arc::clone(&user));
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }

    pub fn mark_inactive(&mut self, id: UserId) -> Result<Arc<User>> {
        use self::users::dsl;
        use diesel::dsl::now;

        info!("Marking user id {} as inactive", id);

        let id: i64 = id.into();
        let user = diesel::update(dsl::users.filter(dsl::user_id.eq(id)))
            .set(dsl::deleted_at.eq(now))
            .get_result::<User>(self.conn)?;

        let user = Arc::new(user);
        self.cache.insert(user.id(), Arc::clone(&user));
        Ok(user)
    }

    #[inline]
    pub fn purge(&mut self) {
        self.cache.clear();
    }
}

impl Debug for UserService<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("UserService")
            .field("conn", &"PgConnection { .. }")
            .finish()
    }
}
