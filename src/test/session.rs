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
use ipnetwork::IpNetwork;

lazy_static! {
    static ref IP_ADDRESS_1: IpNetwork = {
        let ipv6 = "::1".parse().unwrap();
        IpNetwork::new(ipv6, 0).unwrap()
    };
    static ref IP_ADDRESS_2: IpNetwork = {
        let ipv6 = "2004::aa".parse().unwrap();
        IpNetwork::new(ipv6, 0).unwrap()
    };
}

#[test]
fn session_manager() {
    run(|server| task::block_on(session_manager_internal(server)));
}

async fn session_manager_internal(server: &Server) {
    // Setup
    let user_id = server
        .create_user("squirrelbird", "jenny@example.net", "blackmoonhowls")
        .await
        .expect("Unable to create user");

    // Login
    let error = server
        .try_login(user_id, "letmein", *IP_ADDRESS_2)
        .await
        .expect_err("Allowed invalid login");

    match error {
        Error::AuthenticationFailed => (),
        _ => panic!("Error wasn't invalid username or password"),
    }

    let error = server
        .try_login(user_id, "backmonhowl", *IP_ADDRESS_1)
        .await
        .expect_err("Allowed invalid login");

    match error {
        Error::AuthenticationFailed => (),
        _ => panic!("Error wasn't invalid username or password"),
    }

    server
        .try_login(user_id, "blackmoonhowls", *IP_ADDRESS_1)
        .await
        .expect("Unable to login");

    // Check all login attempts
    let date = NaiveDate::from_ymd(2001, 1, 1).and_hms(6, 0, 0);
    let attempts = server
        .get_login_attempts(user_id, DateTime::from_utc(date, Utc))
        .await
        .expect("Unable to get login attempts");

    assert_eq!(attempts.len(), 3);

    let first = &attempts[0];
    let second = &attempts[1];
    let third = &attempts[2];

    assert_eq!(first.user_id(), user_id);
    assert_eq!(first.ip_address(), *IP_ADDRESS_2);
    assert_eq!(first.success(), false);

    assert_eq!(second.user_id(), user_id);
    assert_eq!(second.ip_address(), *IP_ADDRESS_1);
    assert_eq!(second.success(), false);

    assert_eq!(third.user_id(), user_id);
    assert_eq!(third.ip_address(), *IP_ADDRESS_1);
    assert_eq!(third.success(), true);
}
