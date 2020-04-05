/*
 * models/page.rs
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

#[cfg(feature = "ftml-compat")]
use crate::{models::Votes, scoring::Scoring};

#[derive(Serialize, Deserialize, Queryable, Debug, Clone, PartialEq, Eq)]
pub struct Page {
    page_id: PageId,
    wiki_id: WikiId,
    slug: String,
    title: String,
    alt_title: Option<String>,
    tags: Vec<String>,
    created_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
}

impl Page {
    #[inline]
    pub fn id(&self) -> PageId {
        self.page_id
    }

    #[inline]
    pub fn wiki_id(&self) -> WikiId {
        self.wiki_id
    }

    #[inline]
    pub fn slug(&self) -> &str {
        &self.slug
    }

    #[inline]
    pub fn title(&self) -> &str {
        &self.title
    }

    #[inline]
    pub fn alt_title(&self) -> Option<&str> {
        self.alt_title.ref_map(|s| s.as_str())
    }

    #[inline]
    pub fn tags(&self) -> &[String] {
        &self.tags
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
    pub fn exists(&self) -> bool {
        self.deleted_at.is_none()
    }

    #[cfg(feature = "ftml-compat")]
    pub fn into_pageinfo<TScoring: Scoring>(self, votes: Votes) -> ftml::PageInfoOwned {
        let Self {
            title,
            alt_title,
            tags,
            ..
        } = self;
        ftml::PageInfoOwned {
            title,
            alt_title,
            tags,
            header: None,
            subheader: None,
            rating: TScoring::score(&votes),
        }
    }
}
