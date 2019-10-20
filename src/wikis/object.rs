/*
 * wikis/object.rs
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

use chrono::NaiveDateTime;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct WikiId(i64);

impl Into<i64> for WikiId {
    #[inline]
    fn into(self) -> i64 {
        self.0
    }
}

#[derive(Debug)]
pub struct Wiki {
    id: WikiId,
    name: String,
    slug: String,
    created_at: NaiveDateTime,
}

impl Wiki {
    #[inline]
    pub fn from_row((id, name, slug, created_at): (i64, String, String, NaiveDateTime)) -> Self {
        let id = WikiId(id);

        Wiki {
            id,
            name,
            slug,
            created_at,
        }
    }

    #[inline]
    pub fn id(&self) -> WikiId {
        self.id
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn slug(&self) -> &str {
        &self.slug
    }

    #[inline]
    pub fn created_at(&self) -> NaiveDateTime {
        self.created_at
    }
}
