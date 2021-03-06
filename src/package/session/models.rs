/*
 * session/models.rs
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

use crate::schema::{login_attempts, sessions};

#[derive(Debug, Insertable)]
#[table_name = "login_attempts"]
pub struct NewLoginAttempt<'a> {
    pub user_id: Option<i64>,
    pub username_or_email: Option<&'a str>,
    pub remote_address: Option<&'a str>,
    pub success: bool,
}

#[derive(Debug, Insertable)]
#[table_name = "sessions"]
pub struct NewSession {
    pub user_id: i64,
    pub login_attempt_id: i64,
}
