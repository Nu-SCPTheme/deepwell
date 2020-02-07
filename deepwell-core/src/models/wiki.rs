/*
 * models/wiki.rs
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
use chrono::Duration;

#[derive(Serialize, Deserialize, Queryable, Debug, Clone, PartialEq, Eq)]
pub struct Wiki {
    id: WikiId,
    name: String,
    slug: String,
    domain: String,
    created_at: DateTime<Utc>,
}

impl Wiki {
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
    pub fn domain(&self) -> &str {
        &self.domain
    }

    #[inline]
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

#[derive(Serialize, Deserialize, Queryable, Debug, Clone, PartialEq, Eq)]
pub struct WikiSettings {
    id: WikiId,
    page_lock_duration: i16,
}

impl WikiSettings {
    #[inline]
    pub fn id(&self) -> WikiId {
        self.id
    }

    #[inline]
    pub fn page_lock_duration(&self) -> Duration {
        Duration::seconds(self.page_lock_duration as i64)
    }
}
