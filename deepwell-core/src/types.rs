/*
 * types.rs
 *
 * deepwell-core - Database management and migrations service
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

make_id_type!(login_attempt, LoginAttemptId);
make_id_type!(page, PageId);
make_id_type!(rating, RatingId);
make_id_type!(revision, RevisionId);
make_id_type!(user, UserId);
make_id_type!(wiki, WikiId);