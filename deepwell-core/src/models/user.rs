/*
 * models/user.rs
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

use super::prelude::*;

#[derive(Serialize, Debug, Copy, Clone, Default, PartialEq, Eq)]
#[serde(default)]
pub struct UserMetadata<'a> {
    pub name: Option<&'a str>,
    pub email: Option<&'a str>,
    pub author_page: Option<&'a str>,
    pub website: Option<&'a str>,
    pub about: Option<&'a str>,
    pub gender: Option<&'a str>,
    pub location: Option<&'a str>,
}

impl UserMetadata<'_> {
    pub fn to_owned(&self) -> UserMetadataOwned {
        macro_rules! clone {
            ($field:tt) => {
                self.$field.map(|s| s.into())
            };
        }

        UserMetadataOwned {
            name: clone!(name),
            email: clone!(email),
            author_page: clone!(author_page),
            website: clone!(website),
            about: clone!(about),
            gender: clone!(gender),
            location: clone!(location),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(default)]
pub struct UserMetadataOwned {
    pub name: Option<String>,
    pub email: Option<String>,
    pub author_page: Option<String>,
    pub website: Option<String>,
    pub about: Option<String>,
    pub gender: Option<String>,
    pub location: Option<String>,
}

impl UserMetadataOwned {
    pub fn borrow(&self) -> UserMetadata {
        macro_rules! borrow {
            ($field:tt) => {
                self.$field.ref_map(|s| s.as_str())
            };
        }

        UserMetadata {
            name: borrow!(name),
            email: borrow!(email),
            author_page: borrow!(author_page),
            website: borrow!(website),
            about: borrow!(about),
            gender: borrow!(gender),
            location: borrow!(location),
        }
    }
}

#[derive(Serialize, Deserialize, Queryable, Debug, Clone, PartialEq, Eq)]
pub struct User {
    user_id: UserId,
    name: String,
    email: String,
    is_verified: bool,
    is_bot: bool,
    author_page: String,
    website: String,
    about: String,
    gender: String,
    location: String,
    created_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
}

impl User {
    #[inline]
    pub fn id(&self) -> UserId {
        self.user_id
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn email(&self) -> &str {
        &self.email
    }

    #[inline]
    pub fn is_verified(&self) -> bool {
        self.is_verified
    }

    #[inline]
    pub fn is_bot(&self) -> bool {
        self.is_bot
    }

    #[inline]
    pub fn author_page(&self) -> &str {
        &self.author_page
    }

    #[inline]
    pub fn website(&self) -> &str {
        &self.website
    }

    #[inline]
    pub fn about(&self) -> &str {
        &self.about
    }

    #[inline]
    pub fn gender(&self) -> &str {
        &self.gender
    }

    #[inline]
    pub fn location(&self) -> &str {
        &self.location
    }

    #[inline]
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    #[inline]
    pub fn deleted_at(&self) -> Option<DateTime<Utc>> {
        self.deleted_at
    }

    #[inline]
    pub fn is_active(&self) -> bool {
        self.deleted_at.is_none()
    }
}
