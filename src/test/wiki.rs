/*
 * test/wiki.rs
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
fn wiki_service() {
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
