/*
 * test/password.rs
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
fn password_service() {
    run(|server| task::block_on(password_service_internal(server)));
}

async fn password_service_internal(srv: &Server) {
    macro_rules! good_password {
        ($user_id:expr, $password:expr) => {
            srv.validate_user_password($user_id, $password)
                .expect("Password doesn't match")
        };
    }

    macro_rules! bad_password {
        ($user_id:expr, $password:expr) => {
            match srv.validate_user_password($user_id, $password) {
                Err(Error::AuthenticationFailed) => (),
                Err(error) => panic!("Unexpected error: {}", error),
                Ok(_) => panic!("Password matched when it shouldn't have"),
            }
        };
    }

    let bad_user_id = UserId::from_raw(999);
    bad_password!(bad_user_id, "blackmoonhowls");
    bad_password!(bad_user_id, "rustybirb1");
    bad_password!(bad_user_id, "letmein");

    let user_id = srv
        .create_user("squirrelbird", "jenny@example.net", "blackmoonhowls")
        .await
        .expect("Unable to create user");

    good_password!(user_id, "blackmoonhowls");
    bad_password!(user_id, "blackmonhowls");
    bad_password!(user_id, "rustybirb1");
    bad_password!(user_id, "letmein");

    srv.set_user_password(user_id, "rustybirb1")
        .expect("Unable to set new password");

    bad_password!(user_id, "blackmoonhowls");
    good_password!(user_id, "rustybirb1");
    bad_password!(user_id, "letmein");
}

#[test]
fn password_default() {
    run(|server| task::block_on(password_default_internal(server)));
}

async fn password_default_internal(srv: &Server) {
    macro_rules! bad_password {
        ($user_id:expr, $password:expr) => {{
            let user_id = UserId::from_raw($user_id);

            match srv.validate_user_password(user_id, $password) {
                Err(Error::AuthenticationFailed) => (),
                Err(error) => panic!("Unexpected error: {}", error),
                Ok(_) => panic!("Password matched when it shouldn't have"),
            }
        }};
    }

    bad_password!(0, "blackmoon");
    bad_password!(1, "blackmoon");
    bad_password!(2, "blackmoon");
    bad_password!(3, "blackmoon");
    bad_password!(4, "blackmoon");
    bad_password!(5, "blackmoon");
}
