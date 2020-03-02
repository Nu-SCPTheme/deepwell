/*
 * roles.rs
 *
 * deepwell-core - Database management and migrations service
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Role {
    Guest,
    Member,
    Staff,
    Moderator,
    Administrator,
}

impl Role {
    pub fn fixed_name(self) -> &'static str {
        match self {
            Role::Guest => "guest",
            Role::Member => "member",
            Role::Staff => "staff",
            Role::Moderator => "moderator",
            Role::Administrator => "admin",
        }
    }
}
