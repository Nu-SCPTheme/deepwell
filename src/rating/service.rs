/*
 * rating/service.rs
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

use crate::service_prelude::*;

make_id_type!(RatingId);

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

pub struct RatingService {
    conn: Arc<PgConnection>,
}

impl RatingService {
    #[inline]
    pub fn new(conn: &Arc<PgConnection>) -> Self {
        let conn = Arc::clone(conn);

        RatingService { conn }
    }

    pub fn get_rating(&self, page_id: PageId) -> Result<Rating> {
        use diesel::dsl::{count, sum};
        use std::convert::TryInto;

        info!("Starting transaction to get rating for page ID {}", page_id);

        // Right now we use SUM() as a simple scoring system, for compatibility with Wikidot.
        // However it's not the best scoring metric and may be adjusted later.
        //
        // Similarly, it might make sense to turn this into a raw query.

        self.conn.transaction::<_, Error, _>(|| {
            let id: i64 = page_id.into();
            let score = ratings::table
                .filter(ratings::page_id.eq(id))
                .select(sum(ratings::rating))
                .first::<Option<i64>>(&*self.conn)?
                .ok_or_else(|| Error::StaticMsg("inconsistency between pages and ratings table"))?;

            let votes = ratings::table
                .filter(ratings::page_id.eq(id))
                .select(count(ratings::user_id))
                .first::<i64>(&*self.conn)?
                .try_into()
                .map_err(|_| Error::StaticMsg("number of votes doesn't fit into u32"))?;

            Ok(Rating { score, votes })
        })
    }
}
