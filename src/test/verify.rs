/*
 * test/verify.rs
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

#[test]
fn verify() {
    run(|server| task::block_on(verify_internal(server)));
}

async fn verify_internal(server: &Server) {
    // Test verify_token
    let user_id = create_user(server).await;
    let user = server
        .get_user_from_id(user_id)
        .await
        .expect("Couldn't find user");

    assert_eq!(user.is_verified(), false, "User is verified on creation");

    let token = server
        .new_verification_token(user_id)
        .await
        .expect("Unable to generate verification token");

    server
        .verify_token(&token)
        .await
        .expect("Unable to verify user with token");

    let user = server
        .get_user_from_id(user_id)
        .await
        .expect("Couldn't find user");

    assert_eq!(user.is_verified(), true, "User is not verified after token");

    // Test verify_user
    let user_id = create_user(server).await;
    let user = server
        .get_user_from_id(user_id)
        .await
        .expect("Couldn't find user");

    assert_eq!(user.is_verified(), false, "User is verified on creation");

    server
        .verify_user(user_id)
        .await
        .expect("Unable to verify user directly");

    let user = server
        .get_user_from_id(user_id)
        .await
        .expect("Couldn't find user");

    assert_eq!(user.is_verified(), true, "User is not verified after token");
}
