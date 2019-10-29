/*
 * auth/models.rs
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

use crate::schema::passwords;

#[derive(Debug, Insertable, AsChangeset)]
#[table_name = "passwords"]
pub struct NewPassword<'a> {
    pub user_id: i64,
    pub hash: &'a [u8],
    pub salt: &'a [u8],
    pub logn: i16,
    pub param_r: i32,
    pub param_p: i32,
}
