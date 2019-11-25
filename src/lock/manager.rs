/*
 * lock/manager.rs
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

use super::models::NewPageLock;
use crate::manager_prelude::*;
use crate::schema::page_locks;
use crate::utils::rows_to_result;

pub struct LockManager {
    conn: Arc<PgConnection>,
}

impl LockManager {
    #[inline]
    pub fn new(conn: &Arc<PgConnection>) -> Self {
        let conn = Arc::clone(conn);

        LockManager { conn }
    }

    pub async fn invalidate_expired(&self) -> Result<usize> {
        use diesel::dsl::now;

        info!("Invalidating all expired page locks");

        let rows = diesel::delete(page_locks::table)
            .filter(page_locks::dsl::locked_until.lt(now))
            .execute(&*self.conn)?;

        Ok(rows)
    }

    pub async fn check(&self, page_id: PageId) -> Result<()> {
        debug!("Checking if a page lock exists for page ID {}", page_id);

        let id: i64 = page_id.into();
        let result = page_locks::table
            .filter(page_locks::dsl::page_id.eq(id))
            .select(page_locks::dsl::user_id)
            .first::<UserId>(&*self.conn)
            .optional()?;

        match result {
            None => Ok(()),
            Some(user_id) => Err(Error::PageLocked(user_id)),
        }
    }

    pub async fn add(
        &self,
        page_id: PageId,
        user_id: UserId,
        lock_duration: chrono::Duration,
    ) -> Result<()> {
        debug!(
            "Creating page lock for page ID {} by user ID {}",
            page_id, user_id,
        );

        let model = NewPageLock {
            page_id: page_id.into(),
            user_id: user_id.into(),
            locked_until: Utc::now() + lock_duration,
        };

        diesel::insert_into(page_locks::table)
            .values(&model)
            .execute(&*self.conn)?;

        Ok(())
    }

    pub async fn remove(&self, page_id: PageId, user_id: UserId) -> Result<bool> {
        debug!(
            "Removing page lock for page ID {} by user ID {}",
            page_id, user_id
        );

        let page_id: i64 = page_id.into();
        let user_id: i64 = user_id.into();

        let rows = diesel::delete(page_locks::table)
            .filter(page_locks::dsl::page_id.eq(page_id))
            .filter(page_locks::dsl::user_id.eq(user_id))
            .execute(&*self.conn)?;

        Ok(rows_to_result(rows))
    }
}

impl_async_transaction!(LockManager);

impl Debug for LockManager {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("LockManager")
            .field("conn", &"PgConnection { .. }")
            .finish()
    }
}
