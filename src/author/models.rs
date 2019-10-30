/*
 * author/models.rs
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

use crate::schema::authors;
use crate::StdResult;
use chrono::NaiveDateTime;
use std::convert::TryFrom;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AuthorType {
    Author,
    Rewrite,
    Translator,
    Maintainer,
}

impl Into<&'static str> for AuthorType {
    fn into(self) -> &'static str {
        use self::AuthorType::*;

        match self {
            Author => "author",
            Rewrite => "rewrite",
            Translator => "translator",
            Maintainer => "maintainer",
        }
    }
}

impl TryFrom<&'_ str> for AuthorType {
    type Error = ();

    fn try_from(value: &str) -> StdResult<Self, ()> {
        let case = match value {
            "author" => AuthorType::Author,
            "rewrite" => AuthorType::Rewrite,
            "translator" => AuthorType::Translator,
            "maintainer" => AuthorType::Maintainer,
            _ => return Err(()),
        };

        Ok(case)
    }
}

#[derive(Debug, Insertable, AsChangeset)]
#[table_name = "authors"]
pub struct NewAuthor {
    pub page_id: i64,
    pub user_id: i64,
    pub author_type: &'static str,
    pub written_at: Option<NaiveDateTime>,
}
