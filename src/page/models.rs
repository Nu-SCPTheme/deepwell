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

use crate::schema::{pages, revisions, tag_history};

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
    pub alt_title: Option<Option<&'a str>>,
}

#[derive(Debug, Insertable)]
#[table_name = "revisions"]
pub struct NewRevision<'a> {
    pub page_id: i64,
    pub user_id: i64,
    pub message: &'a str,
    pub git_commit: &'a [u8],
    pub change_type: &'a str,
}

#[derive(Debug, Insertable)]
#[table_name = "tag_history"]
pub struct NewTagChange<'a> {
    pub revision_id: i64,
    pub added_tags: &'a [&'a str],
    pub removed_tags: &'a [&'a str],
}
