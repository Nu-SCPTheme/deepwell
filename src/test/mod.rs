/*
 * test/mod.rs
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

extern crate color_backtrace;
extern crate tempfile;

mod authors;
mod login;
mod page;
mod password;
mod session;
mod tags;
mod user;
mod verify;
mod wiki;

use self::prelude::*;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::env;
use tempfile::tempdir;

mod prelude {
    pub use super::{create_user, create_user_full, run};
    pub use crate::prelude::*;
    pub use async_std::task;
    pub use either::*;
}

pub fn run(f: fn(&Server)) {
    color_backtrace::install();

    let database_url = &env::var("DATABASE_URL").expect("No DATABASE_URL specified!");
    let temp_dir = tempdir().expect("Unable to create temp dir");
    let revisions_dir = temp_dir.path().into();

    let config = Config {
        database_url,
        revisions_dir,
        password_blacklist: None,
    };

    let server = Server::new(config).expect("Unable to create deepwell server");

    server.test_transaction(|| {
        f(&server);
        Ok(())
    });
}

pub async fn create_user_full(server: &Server, password: &str) -> (UserId, String, String) {
    let username = {
        let mut chars: String = thread_rng().sample_iter(&Alphanumeric).take(16).collect();

        chars.insert_str(0, "user_");
        chars
    };

    let email = format!("{}@example.com", &username);

    println!("Creating test user '{}'", username);
    let id = server
        .create_user(&username, &email, password)
        .await
        .expect("Unable to create user");

    (id, username, email)
}

pub async fn create_user(server: &Server) -> UserId {
    create_user_full(server, "defaultpasswordhere2").await.0
}
