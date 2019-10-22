/*
 * page/models.rs
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

use crate::schema::pages;

#[derive(Debug, Insertable)]
#[table_name = "pages"]
pub struct NewPage<'a> {
    pub wiki_id: i64,
    pub slug: &'a str,
    pub title: &'a str,
    pub alt_title: Option<&'a str>,
}

#[derive(Debug, AsChangeset)]
#[table_name = "pages"]
pub struct UpdatePage<'a> {
    pub slug: Option<&'a str>,
    pub title: Option<&'a str>,
    pub alt_title: Option<&'a str>,
}
