/*
 * rating/manager.rs
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

use super::{NewRating, NewRatingHistory};
use crate::manager_prelude::*;
use crate::utils::rows_to_result;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Rating {
    score: i64,
    votes: u32,
}

impl Rating {
    #[inline]
    pub fn score(&self) -> i64 {
        self.score
    }

    #[inline]
    pub fn votes(&self) -> u32 {
        self.votes
    }
}

#[derive(Serialize, Deserialize, Queryable, Debug, Clone, PartialEq, Eq)]
pub struct RatingHistory {
    rating_id: RatingId,
    page_id: PageId,
    user_id: UserId,
    created_at: DateTime<Utc>,
    rating: Option<i16>,
}

impl RatingHistory {
    #[inline]
    pub fn id(&self) -> RatingId {
        self.rating_id
    }

    #[inline]
    pub fn page_id(&self) -> PageId {
        self.page_id
    }

    #[inline]
    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    #[inline]
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    #[inline]
    pub fn rating(&self) -> Option<i16> {
        self.rating
    }
}

pub struct RatingManager {
    conn: Arc<PgConnection>,
}

impl RatingManager {
    #[inline]
    pub fn new(conn: &Arc<PgConnection>) -> Self {
        let conn = Arc::clone(conn);

        RatingManager { conn }
    }

    pub async fn get_rating(&self, page_id: PageId) -> Result<Rating> {
        use diesel::dsl::{count, sum};
        use std::convert::TryInto;

        info!("Getting rating information for page ID {}", page_id);

        // Right now we use SUM() as a simple scoring system, for compatibility with Wikidot.
        // However it's not the best scoring metric and may be adjusted later.
        //
        // Similarly, it might make sense to turn this into a raw query.

        self.transaction(async {
            let id: i64 = page_id.into();
            let score = ratings::table
                .filter(ratings::page_id.eq(id))
                .select(sum(ratings::rating))
                .first::<Option<i64>>(&*self.conn)?
                .unwrap_or(0);

            let votes = ratings::table
                .filter(ratings::page_id.eq(id))
                .select(count(ratings::user_id))
                .first::<i64>(&*self.conn)?
                .try_into()
                .map_err(|_| Error::StaticMsg("number of votes doesn't fit into u32"))?;

            Ok(Rating { score, votes })
        })
        .await
    }

    pub async fn set(&self, page_id: PageId, user_id: UserId, rating: i16) -> Result<RatingId> {
        self.transaction(async {
            let model = NewRating {
                page_id: page_id.into(),
                user_id: user_id.into(),
                rating,
            };

            trace!("Inserting rating into rating table");
            diesel::insert_into(ratings::table)
                .values(&model)
                .on_conflict((ratings::dsl::page_id, ratings::dsl::user_id))
                .do_update()
                .set(ratings::dsl::rating.eq(rating))
                .execute(&*self.conn)?;

            trace!("Inserting rating into rating history");
            let model = NewRatingHistory::from(model);
            let rating_id = diesel::insert_into(ratings_history::table)
                .values(&model)
                .returning(ratings_history::dsl::rating_id)
                .get_result::<RatingId>(&*self.conn)?;

            Ok(rating_id)
        })
        .await
    }

    pub async fn remove(&self, page_id: PageId, user_id: UserId) -> Result<Option<RatingId>> {
        self.transaction(async {
            let page_id: i64 = page_id.into();
            let user_id: i64 = user_id.into();

            trace!("Deleting rating from rating table");
            let rows = diesel::delete(ratings::table)
                .filter(ratings::page_id.eq(page_id))
                .filter(ratings::user_id.eq(user_id))
                .execute(&*self.conn)?;

            if !rows_to_result(rows) {
                return Ok(None);
            }

            let model = NewRatingHistory {
                page_id,
                user_id,
                rating: None,
            };

            let rating_id = diesel::insert_into(ratings_history::table)
                .values(&model)
                .returning(ratings_history::dsl::rating_id)
                .get_result::<RatingId>(&*self.conn)?;

            Ok(Some(rating_id))
        })
        .await
    }

    pub async fn get_history(
        &self,
        page_id: PageId,
        user_id: UserId,
    ) -> Result<Vec<RatingHistory>> {
        debug!(
            "Getting rating history for page ID {} / user ID {}",
            page_id, user_id,
        );

        let page_id: i64 = page_id.into();
        let user_id: i64 = user_id.into();

        let result = ratings_history::table
            .filter(ratings_history::page_id.eq(page_id))
            .filter(ratings_history::user_id.eq(user_id))
            .order_by(ratings_history::created_at.asc())
            .load::<RatingHistory>(&*self.conn)?;

        Ok(result)
    }

    pub async fn get_history_latest(
        &self,
        page_id: PageId,
        user_id: UserId,
    ) -> Result<Option<RatingHistory>> {
        info!(
            "Getting last rating history entry for page ID {} / user ID {}",
            page_id, user_id,
        );

        let page_id: i64 = page_id.into();
        let user_id: i64 = user_id.into();

        let result = ratings_history::table
            .filter(ratings_history::page_id.eq(page_id))
            .filter(ratings_history::user_id.eq(user_id))
            .order_by(ratings_history::created_at.asc())
            .first::<RatingHistory>(&*self.conn)
            .optional()?;

        Ok(result)
    }

    pub async fn get_history_entry(&self, rating_id: RatingId) -> Result<Option<RatingHistory>> {
        info!("Getting rating history entry for ID {}", rating_id);

        let id: i64 = rating_id.into();
        let result = ratings_history::table
            .find(id)
            .first::<RatingHistory>(&*self.conn)
            .optional()?;

        Ok(result)
    }
}

impl_async_transaction!(RatingManager);

impl Debug for RatingManager {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PasswordManager")
            .field("conn", &"PgConnection { .. }")
            .finish()
    }
}
