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
    run(|handle| task::block_on(user_service_internal(handle)));
}

async fn user_service_internal(handle: &Handle) {
    let user_id = handle
        .create_user("squirrelbird", "jenny@example.net", "blackmoonhowls")
        .await
        .expect("Unable to create user");

    let metadata = UserMetadata {
        name: Some("Jenny Person"),
        email: None,
        author_page: Some("http://www.scp-wiki.net/authors-pages"),
        website: None,
        about: Some("A totally real person who writes"),
        gender: Some("FEMALE"),
        location: Some("Earth"),
    };

    handle
        .edit_user(user_id, metadata)
        .await
        .expect("Unable to edit user");

    handle
        .verify_user(user_id)
        .await
        .expect("Unable to mark user as verified");

    handle
        .mark_user_inactive(user_id)
        .await
        .expect("Unable to mark user as inactive");

    handle
        .mark_user_active(user_id)
        .await
        .expect("Unable to reactivate user");

    let user_id_2 = handle
        .create_user("otheruser", "jeremy@example.net", "superstrongpassword")
        .await
        .expect("Unable to create second user");

    let metadata = UserMetadata {
        name: None,
        email: None,
        author_page: None,
        website: None,
        about: Some("test user 2"),
        gender: Some("non-binary"),
        location: Some("Earth"),
    };

    handle
        .edit_user(user_id_2, metadata)
        .await
        .expect("Unable to edit second user");

    let user_1 = handle
        .get_user_from_name("Jenny Person")
        .await
        .expect("Unable to get user by username")
        .expect("No such user with this name");

    let user_2 = handle
        .get_user_from_id(user_id_2)
        .await
        .expect("Unable to get user from ID");

    let users = handle
        .get_users_from_ids(&[user_id, UserId::from_raw(9999), user_id_2])
        .await
        .expect("Unable to get multiple users");

    assert_eq!(users, vec![Some(user_1), None, Some(user_2)]);
}
