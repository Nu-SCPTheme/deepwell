/*
 * page/models.rs
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

use crate::schema::{pages, revisions, tag_history};
use crate::StdResult;
use std::convert::TryFrom;

type Nullable<T> = Option<T>;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ChangeType {
    Create,
    Modify,
    Delete,
    Restore,
    Rename,
    Undo,
    Tags,
}

impl ChangeType {
    // For git commit messages
    pub fn verb(&self) -> &'static str {
        use self::ChangeType::*;

        match self {
            Create => "created",
            Modify => "modified",
            Delete => "deleted",
            Restore => "restored",
            Rename => "renamed",
            Undo => "reverted",
            Tags => "tags",
        }
    }
}

impl Into<&'static str> for ChangeType {
    // The "enum name" used to store in the database
    // Cannot change without a migration
    fn into(self) -> &'static str {
        use self::ChangeType::*;

        match self {
            Create => "create",
            Modify => "modify",
            Delete => "delete",
            Restore => "restore",
            Rename => "rename",
            Undo => "undo",
            Tags => "tags",
        }
    }
}

impl TryFrom<&'_ str> for ChangeType {
    type Error = ();

    fn try_from(value: &str) -> StdResult<Self, ()> {
        let case = match value {
            "create" => ChangeType::Create,
            "modify" => ChangeType::Modify,
            "delete" => ChangeType::Delete,
            "restore" => ChangeType::Restore,
            "rename" => ChangeType::Rename,
            "tags" => ChangeType::Tags,
            _ => return Err(()),
        };

        Ok(case)
    }
}

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
    pub alt_title: Option<Nullable<&'a str>>,
}

impl UpdatePage<'_> {
    pub fn has_changes(&self) -> bool {
        self.slug.is_some() || self.title.is_some() || self.alt_title.is_some()
    }
}

#[derive(Debug, Insertable)]
#[table_name = "revisions"]
pub struct NewRevision<'a> {
    pub page_id: i64,
    pub user_id: i64,
    pub message: &'a str,
    pub git_commit: &'a str,
    pub change_type: &'a str,
}

#[derive(Debug, Insertable)]
#[table_name = "tag_history"]
pub struct NewTagChange<'a> {
    pub revision_id: i64,
    pub added_tags: &'a [&'a str],
    pub removed_tags: &'a [&'a str],
}
