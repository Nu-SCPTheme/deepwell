/*
 * test/user.rs
 *
 * deepwell - Database management and migrations service
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

macro_rules! check_err {
    ($error:expr, $expected:pat) => {
        match $error {
            $expected => (),
            _ => panic!("Error doesn't match"),
        }
    };
}

#[test]
fn users() {
    run(|server| task::block_on(users_internal(server)));
}

async fn users_internal(server: &Server) {
    let user_id = create_user(server).await;
    let metadata = UserMetadata {
        name: Some("Jenny Person"),
        email: None,
        user_page: Some("http://www.scp-wiki.net/authors-pages"),
        website: None,
        about: Some("A totally real person who writes"),
        gender: Some("FEMALE"),
        location: Some("Earth"),
    };

    server
        .edit_user(user_id, metadata)
        .await
        .expect("Unable to edit user");

    server
        .verify_user(user_id)
        .await
        .expect("Unable to mark user as verified");

    server
        .mark_user_inactive(user_id)
        .await
        .expect("Unable to mark user as inactive");

    server
        .mark_user_active(user_id)
        .await
        .expect("Unable to reactivate user");

    let user_id_2 = create_user(server).await;
    let metadata = UserMetadata {
        name: None,
        email: None,
        user_page: None,
        website: None,
        about: Some("test user 2"),
        gender: Some("non-binary"),
        location: Some("Earth"),
    };

    server
        .edit_user(user_id_2, metadata)
        .await
        .expect("Unable to edit second user");

    let user_1 = server
        .get_user_from_name("Jenny Person")
        .await
        .expect("Unable to get user by username")
        .expect("No such user with this name");

    let user_2 = server
        .get_user_from_id(user_id_2)
        .await
        .expect("Unable to get user from ID")
        .expect("Created user not found");

    let invalid = UserId::from_raw(-1);
    let users = server
        .get_users_from_ids(&[user_id, invalid, user_id_2])
        .await
        .expect("Unable to get multiple users");

    assert_eq!(users, vec![Some(user_1), None, Some(user_2)]);

    let error = server
        .get_users_from_ids(&vec![invalid; 198])
        .await
        .expect_err("Able to fetch over 100 users");

    check_err!(error, Error::RequestTooLarge(198, 100));
}

#[test]
fn users_conflict() {
    run(|server| task::block_on(users_conflict_internal(server)));
}

async fn users_conflict_internal(server: &Server) {
    let user_id_1 = create_user(server).await;
    let user_id_2 = create_user(server).await;

    // Set initial user info
    server
        .edit_user(
            user_id_1,
            UserMetadata {
                name: Some("conflictTest joe"),
                email: Some("joe@example.net"),
                ..UserMetadata::default()
            },
        )
        .await
        .expect("Unable to edit user initially");

    server
        .edit_user(
            user_id_2,
            UserMetadata {
                name: Some("conflictTest jim"),
                email: Some("jim@example.net"),
                ..UserMetadata::default()
            },
        )
        .await
        .expect("Unable to edit user initially");

    // Check conflicts with username
    let error = server
        .edit_user(
            user_id_1,
            UserMetadata {
                name: Some("conflictTest jim"),
                ..UserMetadata::default()
            },
        )
        .await
        .expect_err("Conflicted username edit succeeded");

    check_err!(error, Error::UserNameExists);

    // Try changing to same username
    server
        .edit_user(
            user_id_1,
            UserMetadata {
                name: Some("conflictTest joe"),
                ..UserMetadata::default()
            },
        )
        .await
        .expect("Unable to set username to equivalent value");

    // Check conflicts with email
    let error = server
        .edit_user(
            user_id_1,
            UserMetadata {
                email: Some("jim@example.net"),
                ..UserMetadata::default()
            },
        )
        .await
        .expect_err("Conflicted username edit succeeded");

    check_err!(error, Error::UserEmailExists);

    // Try changing to same email
    server
        .edit_user(
            user_id_2,
            UserMetadata {
                email: Some("jim@example.net"),
                ..UserMetadata::default()
            },
        )
        .await
        .expect("Unable to set email to equivalent value");
}
