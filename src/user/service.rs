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
use parking_lot::Mutex;
use std::sync::Arc;

pub struct UserService<'d> {
    conn: &'d PgConnection,
    cache: Mutex<LruCache<UserId, Arc<User>>>,
}

impl<'d> UserService<'d> {
    #[inline]
    pub fn new(conn: &'d PgConnection, cache_size: usize) -> Self {
        let cache = Mutex::new(LruCache::with_capacity(cache_size));

        UserService { conn, cache }
    }

    pub fn create(&self, name: &str, email: &str) -> Result<Arc<User>> {
        info!(
            "Creating new user with name '{}' with email '{}'",
            name, email
        );

        let mut cache = self.cache.lock();
        let model = NewUser { name, email };
        let user = diesel::insert_into(users::table)
            .values(&model)
            .get_result::<User>(self.conn)?;

        let user = Arc::new(user);
        cache.insert(user.id(), Arc::clone(&user));
        Ok(user)
    }

    pub fn get(&self, id: UserId) -> Result<Option<Arc<User>>> {
        info!("Getting user for id {}", id);

        let mut cache = self.cache.lock();
        if let Some(user) = cache.get(&id) {
            debug!("Found user in cache");
            return Ok(Some(Arc::clone(user)));
        }

        let id: i64 = id.into();
        let result = users::table.find(id).first::<User>(self.conn).optional()?;
        match result {
            Some(user) => {
                let user = Arc::new(user);
                cache.insert(user.id(), Arc::clone(&user));
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }

    pub fn edit(&self, id: UserId, model: UpdateUser) -> Result<Arc<User>> {
        use self::users::dsl;

        info!("Editing user id {}, changes: {:?}", id, model);

        let id: i64 = id.into();
        let mut cache = self.cache.lock();
        let user = diesel::update(dsl::users.filter(dsl::user_id.eq(id)))
            .set(model)
            .get_result::<User>(self.conn)?;

        let user = Arc::new(user);
        cache.insert(user.id(), Arc::clone(&user));
        Ok(user)
    }

    pub fn mark_inactive(&self, id: UserId) -> Result<Arc<User>> {
        use self::users::dsl;
        use diesel::dsl::now;

        info!("Marking user id {} as inactive", id);

        let id: i64 = id.into();
        let mut cache = self.cache.lock();
        let user = diesel::update(dsl::users.filter(dsl::user_id.eq(id)))
            .set(dsl::deleted_at.eq(now))
            .get_result::<User>(self.conn)?;

        let user = Arc::new(user);
        cache.insert(user.id(), Arc::clone(&user));
        Ok(user)
    }

    #[inline]
    pub fn purge(&self) {
        self.cache.lock().clear();
    }
}

impl Debug for UserService<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("UserService")
            .field("conn", &"PgConnection { .. }")
            .field("cache", &"LruCache { .. }")
            .finish()
    }
}
