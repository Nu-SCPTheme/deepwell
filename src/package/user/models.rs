/*
 * user/models.rs
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

use crate::schema::{user_verification, users};
use chrono::prelude::*;

#[derive(Debug, Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub name: &'a str,
    pub email: &'a str,
}

type Nullable<T> = Option<T>;

#[derive(Debug, Default, AsChangeset)]
#[table_name = "users"]
pub struct UpdateUser<'a> {
    pub name: Option<&'a str>,
    pub email: Option<&'a str>,
    pub is_verified: Option<bool>,
    pub user_page: Option<&'a str>,
    pub website: Option<&'a str>,
    pub about: Option<&'a str>,
    pub gender: Option<&'a str>,
    pub location: Option<&'a str>,
    pub deleted_at: Option<Nullable<DateTime<Utc>>>,
}

impl UpdateUser<'_> {
    pub fn has_changes(&self) -> bool {
        self.name.is_some()
            || self.email.is_some()
            || self.is_verified.is_some()
            || self.user_page.is_some()
            || self.website.is_some()
            || self.about.is_some()
            || self.gender.is_some()
            || self.location.is_some()
            || self.deleted_at.is_some()
    }
}

#[derive(Debug, Insertable)]
#[table_name = "user_verification"]
pub struct NewUserVerification<'a> {
    pub user_id: i64,
    pub token: &'a str,
}
