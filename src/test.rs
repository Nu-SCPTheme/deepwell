/*
 * test.rs
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

extern crate color_backtrace;
extern crate tempfile;

use crate::prelude::*;
use std::env;
use tempfile::tempdir;

fn run<F: FnOnce(&Server)>(f: F) {
    color_backtrace::install();

    let database_url = &env::var("DATABASE_URL").expect("No DATABASE_URL specified!");
    let temp_dir = tempdir().expect("Unable to create temp dir");
    let revisions_dir = temp_dir.path().into();

    let config = ServerConfig {
        database_url,
        revisions_dir,
        password_blacklist: None,
    };

    let server = Server::new(config).expect("Unable to create server");

    server.test_transaction(|| {
        f(&server);
        Ok(())
    });
}

#[test]
fn test_wiki() {
    run(|srv| {
        let wiki_id = srv
            .create_wiki("Test Wiki", "test", "example.com")
            .expect("Unable to create wiki");

        srv.rename_wiki(wiki_id, "NUTTEST")
            .expect("Unable to rename wiki");

        srv.set_wiki_domain(wiki_id, "example.org")
            .expect("Unable to change domain");

        {
            let id = srv.get_wiki_id("test").expect("Couldn't find wiki");
            assert_eq!(id, wiki_id);
        }

        {
            let err = srv.get_wiki_id("nonexistent").expect_err("Found wiki");
            match err {
                Error::WikiNotFound => (),
                _ => panic!("Error doesn't match"),
            }
        }
    });
}

#[test]
fn test_user() {
    run(|srv| {
        let user_id = srv
            .create_user("squirrelbird", "jenny@example.net", "blackmoonhowls")
            .expect("Unable to create user");

        srv.edit_user(
            user_id,
            Some("Jenny User"),
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
    });
}
