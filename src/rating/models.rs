/*
 * rating/models.rs
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

use crate::schema::{ratings, ratings_history};

#[derive(Debug, Insertable)]
#[table_name = "ratings"]
pub struct NewRating {
    pub page_id: i64,
    pub user_id: i64,
    pub rating: i16,
}

#[derive(Debug, Insertable)]
#[table_name = "ratings_history"]
pub struct NewRatingHistory {
    pub page_id: i64,
    pub user_id: i64,
    pub rating: i16,
}

impl From<NewRating> for NewRatingHistory {
    fn from(model: NewRating) -> NewRatingHistory {
        let NewRating {
            page_id,
            user_id,
            rating,
        } = model;

        NewRatingHistory {
            page_id,
            user_id,
            rating,
        }
    }
}
