/*
 * author/service.rs
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

use super::{AuthorType, NewAuthor};
use crate::schema::authors;
use crate::service_prelude::*;
use std::convert::TryFrom;

#[derive(Serialize, Deserialize, Queryable, Debug, Clone, PartialEq, Eq)]
pub struct Author {
    user_id: UserId,
    page_id: PageId,
    author_type: String,
    written_at: NaiveDate,
}

impl Author {
    #[inline]
    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    #[inline]
    pub fn page_id(&self) -> PageId {
        self.page_id
    }

    #[inline]
    pub fn author_type(&self) -> AuthorType {
        let value = self.author_type.as_str();

        AuthorType::try_from(value).expect("author type in database invalid")
    }

    #[inline]
    pub fn written_at(&self) -> NaiveDate {
        self.written_at
    }
}

pub struct AuthorService {
    conn: Arc<PgConnection>,
}

impl AuthorService {
    pub fn new(conn: &Arc<PgConnection>) -> Self {
        let conn = Arc::clone(conn);

        AuthorService { conn }
    }

    pub fn get_authors(&self, page_id: PageId) -> Result<Vec<Author>> {
        info!("Getting authors for page id {}", page_id);

        let id: i64 = page_id.into();
        let result = authors::table
            .filter(authors::dsl::page_id.eq(id))
            .load::<Author>(&*self.conn)?;

        Ok(result)
    }

    pub fn set_author(
        &self,
        page_id: PageId,
        user_id: UserId,
        author_type: AuthorType,
        written_at: Option<NaiveDate>,
    ) -> Result<()> {
        debug!(
            "Setting author for page id {} / user id {}",
            page_id, user_id
        );

        let model = NewAuthor {
            page_id: page_id.into(),
            user_id: user_id.into(),
            author_type: author_type.into(),
            written_at,
        };

        diesel::insert_into(authors::table)
            .values(&model)
            .on_conflict((authors::dsl::page_id, authors::dsl::user_id))
            .do_update()
            .set(&model)
            .execute(&*self.conn)?;

        Ok(())
    }
}

impl Debug for AuthorService {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("AuthorService")
            .field("conn", &"PgConnection { .. }")
            .finish()
    }
}
