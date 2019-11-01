/*
 * test/user.rs
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

use super::prelude::*;

#[test]
fn user_service() {
    run(|srv| {
        let user_id = srv
            .create_user("squirrelbird", "jenny@example.net", "blackmoonhowls")
            .expect("Unable to create user");

        srv.edit_user(
            user_id,
            Some("Jenny Person"),
            None,
            Some("http://www.scp-wiki.net/authors-pages"),
            None,
            Some("A totally real person who writes"),
            Some("FEMALE"),
            Some("Earth"),
        )
        .expect("Unable to edit user");

        srv.verify_user(user_id)
            .expect("Unable to mark user as verified");

        srv.mark_user_inactive(user_id)
            .expect("Unable to mark user as inactive");
        srv.mark_user_active(user_id)
            .expect("Unable to reactivate user");

        let user_id_2 = srv
            .create_user("otheruser", "jeremy@example.net", "superstrongpassword")
            .expect("Unable to create second user");

        srv.edit_user(
            user_id_2,
            None,
            None,
            None,
            None,
            Some("test user 2"),
            Some("nb"),
            None,
        )
        .expect("Unable to edit second user");

        let user_1 = srv
            .get_user_from_name("Jenny Person")
            .expect("Unable to get user by username")
            .expect("No such user with this name");
        let user_2 = srv
            .get_user_from_id(user_id_2)
            .expect("Unable to get user from ID");

        let users = srv
            .get_users_from_ids(&[user_id, UserId::from_raw(9999), user_id_2])
            .expect("Unable to get multiple users");
        assert_eq!(users, vec![Some(user_1), None, Some(user_2)]);
    });
}
