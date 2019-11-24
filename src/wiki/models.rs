/*
 * wiki/models.rs
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

use crate::schema::wikis;

#[derive(Debug, Insertable)]
#[table_name = "wikis"]
pub struct NewWiki<'a> {
    pub name: &'a str,
    pub slug: &'a str,
    pub domain: &'a str,
}

#[derive(Debug, Default, AsChangeset)]
#[table_name = "wikis"]
pub struct UpdateWiki<'a> {
    pub name: Option<&'a str>,
    pub domain: Option<&'a str>,
}

impl UpdateWiki<'_> {
    pub fn has_changes(&self) -> bool {
        self.name.is_some() || self.domain.is_some()
    }
}
