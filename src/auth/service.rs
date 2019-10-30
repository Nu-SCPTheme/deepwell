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

use super::{build_blacklist, check_password, new_password};
use crate::schema::passwords;
use crate::service_prelude::*;
use std::collections::HashSet;
use std::convert::TryInto;
use std::path::Path;

const MAX_PASSWORD_LEN: usize = 8192;

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
    password_blacklist: HashSet<String>,
}

impl AuthService {
    pub fn new(conn: &Rc<PgConnection>, blacklist: Option<&Path>) -> Result<Self> {
        let conn = Rc::clone(conn);

        let password_blacklist = match blacklist {
            Some(path) => build_blacklist(path)?,
            None => HashSet::new(),
        };

        Ok(AuthService {
            conn,
            password_blacklist,
        })
    }

    fn verify_password(&self, password: &str) -> Result<()> {
        // To avoid computation-based DOS attacks
        if password.len() > MAX_PASSWORD_LEN {
            return Err(Error::NewPasswordInvalid("password too long"));
        }

        if password.chars().count() < 8 {
            return Err(Error::NewPasswordInvalid(
                "password must be at least 8 characters",
            ));
        }

        if self.password_blacklist.contains(password) {
            return Err(Error::NewPasswordInvalid("password is too common"));
        }

        Ok(())
    }

    pub fn has_password(&self, user_id: UserId) -> Result<bool> {
        let id: i64 = user_id.into();
        let record = passwords::table
            .find(id)
            .select(passwords::dsl::user_id)
            .first::<UserId>(&*self.conn)
            .optional()?;

        Ok(record.is_some())
    }

    pub fn set_password(&self, user_id: UserId, password: &str) -> Result<()> {
        self.verify_password(password)?;

        new_password(user_id, password.as_bytes(), |model| {
            diesel::insert_into(passwords::table)
                .values(&model)
                .on_conflict(passwords::dsl::user_id)
                .do_update()
                .set(&model)
                .execute(&*self.conn)?;

            Ok(())
        })
    }

    pub fn check_password(&self, user_id: UserId, password: &str) -> Result<()> {
        // To avoid computation-based DOS attacks
        if password.len() > MAX_PASSWORD_LEN {
            return Err(Error::AuthenticationFailed);
        }

        let id: i64 = user_id.into();
        let record = passwords::table
            .find(id)
            .first::<Password>(&*self.conn)
            .optional()?;

        let record = record.ok_or(Error::AuthenticationFailed)?;
        if check_password(&record, password.as_bytes()) {
            Ok(())
        } else {
            Err(Error::AuthenticationFailed)
        }
    }
}

impl Debug for AuthService {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("AuthService")
            .field("conn", &"PgConnection { .. }")
            .finish()
    }
}
