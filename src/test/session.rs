/*
 * test/session.rs
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
    ($error:expr) => {
        match $error {
            Error::NotLoggedIn => (),
            _ => panic!("Error wasn't not logged in"),
        }
    };
}

#[test]
fn session_manager() {
    run(|server| {
        task::block_on(session_manager_internal(server));
    });
}

async fn session_manager_internal(server: &Server) {
    // Setup
    let user_id = server
        .create_user("squirrelbird", "jenny@example.net", "blackmoonhowls")
        .await
        .expect("Unable to create user");

    // Login with user ID
    let session_1 = server
        .try_login_id(user_id, "blackmoonhowls", None)
        .await
        .expect("Unable to login");

    assert_eq!(user_id, session_1.user_id());

    server
        .check_session(session_1.session_id(), session_1.user_id())
        .await
        .expect("Session was invalid");

    // Login with username
    let session_2 = server
        .try_login("squirrelbird", "blackmoonhowls", None)
        .await
        .expect("Unable to login");

    assert_eq!(user_id, session_2.user_id());

    server
        .check_session(session_2.session_id(), session_2.user_id())
        .await
        .expect("Session was invalid");

    // Login with email
    let session_3 = server
        .try_login("jenny@example.net", "blackmoonhowls", None)
        .await
        .expect("Unable to login");

    assert_eq!(user_id, session_3.user_id());

    server
        .check_session(session_3.session_id(), session_3.user_id())
        .await
        .expect("Session was invalid");

    // Invalidate session 1
    server
        .end_session(session_1.session_id(), session_1.user_id())
        .await
        .expect("Unable to end session");

    let error = server
        .check_session(session_1.session_id(), session_1.user_id())
        .await
        .expect_err("Session still valid");

    check_err!(error);

    // Invalidate session 2
    server
        .end_session(session_2.session_id(), session_2.user_id())
        .await
        .expect("Unable to end session");

    let error = server
        .check_session(session_2.session_id(), session_2.user_id())
        .await
        .expect_err("Session still valid");

    check_err!(error);

    // Invalidate invalid session
    let error = server
        .end_session(session_1.session_id(), session_1.user_id())
        .await
        .expect_err("Unable to end session");

    check_err!(error);

    // Invalidate session 3
    server
        .end_session(session_3.session_id(), session_3.user_id())
        .await
        .expect("Unable to end session");

    let error = server
        .check_session(session_3.session_id(), session_3.user_id())
        .await
        .expect_err("Session still valid");

    check_err!(error);
}
