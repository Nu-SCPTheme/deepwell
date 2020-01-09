/*
 * server/utils.rs
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

use wikidot_normalize::normalize;

pub fn normalize_slug<S: Into<String>>(slug: S) -> String {
    let mut slug = slug.into();
    normalize(&mut slug);
    slug
}

pub fn to_lowercase<S: Into<String>>(value: S) -> String {
    let mut value = value.into();
    value.make_ascii_lowercase();
    value
}
