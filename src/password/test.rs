/*
 * password/test.rs
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

use super::{check_password, new_password, Password};
use async_std::task;
use deepwell_core::UserId;

#[test]
fn crypto() {
    color_backtrace::install();

    task::block_on(crypto_inner());
}

async fn crypto_inner() {
    // Since we're not actually using Diesel to persist to disk,
    // we have to locally store the values here.

    let user = UserId::from_raw(0);

    let mut hash = Vec::new();
    let mut salt = Vec::new();
    let (mut logn, mut param_r, mut param_p) = (0, 0, 0);

    new_password(user, b"apples and bananas", |model| {
        hash.extend_from_slice(model.hash);
        salt.extend_from_slice(model.salt);

        logn = model.logn;
        param_r = model.param_r;
        param_p = model.param_p;

        Ok(())
    })
    .await
    .unwrap();

    let record = Password::new(user, hash, salt, logn, param_r, param_p);

    macro_rules! check {
        ($password:expr, $expected:expr) => {{
            println!("Checking password: '{}'", $password);
            let actual = check_password(&record, $password.as_bytes()).await;
            assert_eq!(actual, $expected, "Password result mismatch");
        }};
    }

    check!("", false);
    check!("apples and bananas", true);
    check!("apples and banana", false);
}
