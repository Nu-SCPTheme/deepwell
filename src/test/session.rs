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
use chrono::prelude::*;

const IP_ADDRESS_1: Option<&str> = Some("alpha-beta.local");
const IP_ADDRESS_2: Option<&str> = Some("1.1.1.1");
const IP_ADDRESS_3: Option<&str> = None;

macro_rules! check_err {
    ($error:expr) => {
        match $error {
            Error::AuthenticationFailed => (),
            _ => panic!("Error wasn't invalid username or password"),
        }
    };
}

#[test]
fn session_manager_id() {
    run(|server| {
        task::block_on(async {
            let user_id = setup(server).await;
            session_manager_internal_id(server, user_id).await;
        })
    });
}

#[test]
fn session_manager_name() {
    run(|server| {
        task::block_on(async {
            let user_id = setup(server).await;
            session_manager_internal_name(server, user_id).await;
        })
    });
}

#[inline]
fn now() -> DateTime<Utc> {
    DateTime::from(Local::now())
}

async fn setup(server: &Server) -> UserId {
    server
        .create_user("squirrelbird", "jenny@example.net", "blackmoonhowls")
        .await
        .expect("Unable to create user")
}

async fn session_manager_internal_id(server: &Server, user_id: UserId) {
    // Login
    let error = server
        .try_login_id(user_id, "letmein", IP_ADDRESS_2)
        .await
        .expect_err("Allowed invalid login");

    check_err!(error);

    let error = server
        .try_login_id(user_id, "backmonhowl", IP_ADDRESS_1)
        .await
        .expect_err("Allowed invalid login");

    check_err!(error);

    server
        .try_login_id(user_id, "blackmoonhowls", IP_ADDRESS_3)
        .await
        .expect("Unable to login");

    // Check all login attempts
    let attempts = server
        .get_login_attempts(user_id, now())
        .await
        .expect("Unable to get login attempts");

    assert_eq!(attempts.len(), 3);

    let first = &attempts[0];
    let second = &attempts[1];
    let third = &attempts[2];

    assert_eq!(first.user_id(), Some(user_id));
    assert_eq!(first.username_or_email(), None);
    assert_eq!(first.remote_address(), IP_ADDRESS_2);
    assert_eq!(first.success(), false);

    assert_eq!(second.user_id(), Some(user_id));
    assert_eq!(second.username_or_email(), None);
    assert_eq!(second.remote_address(), IP_ADDRESS_1);
    assert_eq!(second.success(), false);

    assert_eq!(third.user_id(), Some(user_id));
    assert_eq!(third.username_or_email(), None);
    assert_eq!(third.remote_address(), IP_ADDRESS_3);
    assert_eq!(third.success(), true);
}

async fn session_manager_internal_name(server: &Server, user_id: UserId) {
    // Login
    let error = server
        .try_login("squirrel", "blackmoonhowls", IP_ADDRESS_3)
        .await
        .expect_err("Allowed invalid login");

    check_err!(error);

    let error = server
        .try_login("squirrelbird", "letmein", IP_ADDRESS_1)
        .await
        .expect_err("Allowed invalid login");

    check_err!(error);

    server
        .try_login("squirrelbird", "blackmoonhowls", IP_ADDRESS_2)
        .await
        .expect("Unable to login");

    let error = server
        .try_login("jenny@gmail.com", "blackmoonhowls", IP_ADDRESS_1)
        .await
        .expect_err("Allowed invalid login");

    check_err!(error);

    let error = server
        .try_login("jenny@example.net", "letmein", IP_ADDRESS_3)
        .await
        .expect_err("Allowed invalid login");

    check_err!(error);

    server
        .try_login("jenny@example.net", "blackmoonhowls", IP_ADDRESS_2)
        .await
        .expect("Unable to login");

    // Check all login attempts
    let attempts = server
        .get_all_login_attempts(now())
        .await
        .expect("Unable to get login attempts");

    assert_eq!(attempts.len(), 6);

    let first = &attempts[0];
    let second = &attempts[1];
    let third = &attempts[2];
    let fourth = &attempts[3];
    let fifth = &attempts[4];
    let sixth = &attempts[5];

    assert_eq!(first.user_id(), None);
    assert_eq!(first.username_or_email(), Some("squirrel"));
    assert_eq!(first.remote_address(), IP_ADDRESS_3);
    assert_eq!(first.success(), false);

    assert_eq!(second.user_id(), Some(user_id));
    assert_eq!(second.username_or_email(), None);
    assert_eq!(second.remote_address(), IP_ADDRESS_1);
    assert_eq!(second.success(), false);

    assert_eq!(third.user_id(), Some(user_id));
    assert_eq!(third.username_or_email(), None);
    assert_eq!(third.remote_address(), IP_ADDRESS_2);
    assert_eq!(third.success(), true);

    assert_eq!(fourth.user_id(), None);
    assert_eq!(fourth.username_or_email(), Some("jenny@gmail.com"));
    assert_eq!(fourth.remote_address(), IP_ADDRESS_1);
    assert_eq!(fourth.success(), false);

    assert_eq!(fifth.user_id(), Some(user_id));
    assert_eq!(fifth.username_or_email(), None);
    assert_eq!(fifth.remote_address(), IP_ADDRESS_3);
    assert_eq!(fifth.success(), false);

    assert_eq!(sixth.user_id(), Some(user_id));
    assert_eq!(sixth.username_or_email(), None);
    assert_eq!(sixth.remote_address(), IP_ADDRESS_2);
    assert_eq!(sixth.success(), true);
}
