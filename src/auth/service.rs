/*
 * auth/service.rs
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

use crate::service_prelude::*;
use std::convert::TryInto;

#[derive(Debug, Queryable)]
pub struct Password {
    user_id: UserId,
    hash: Vec<u8>,
    salt: Vec<u8>,
    logn: i16,
    param_r: i32,
    param_p: i32,
}

impl Password {
    #[cfg(test)]
    #[inline]
    pub fn new(
        user_id: UserId,
        hash: Vec<u8>,
        salt: Vec<u8>,
        logn: i16,
        param_r: i32,
        param_p: i32,
    ) -> Self {
        Password {
            user_id,
            hash,
            salt,
            logn,
            param_r,
            param_p,
        }
    }

    #[inline]
    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    #[inline]
    pub fn hash(&self) -> &[u8] {
        &self.hash
    }

    #[inline]
    pub fn salt(&self) -> &[u8] {
        &self.salt
    }

    #[inline]
    pub fn logn(&self) -> u8 {
        self.logn
            .try_into()
            .expect("Stored log_n field is out of bounds")
    }

    #[inline]
    pub fn param_r(&self) -> u32 {
        self.param_r
            .try_into()
            .expect("Stored param_r field is out of bounds")
    }

    #[inline]
    pub fn param_p(&self) -> u32 {
        self.param_p
            .try_into()
            .expect("Stored param_r field is out of bounds")
    }
}

pub struct AuthService {
    conn: Rc<PgConnection>,
}

impl AuthService {
    pub fn new(conn: &Rc<PgConnection>) -> Self {
        let conn = Rc::clone(conn);

        AuthService { conn }
    }
}

impl Debug for AuthService {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("AuthService")
            .field("conn", &"PgConnection { .. }")
            .finish()
    }
}
