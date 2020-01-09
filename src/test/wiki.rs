/*
 * test/wiki.rs
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
fn wiki_manager() {
    run(|server| task::block_on(wiki_manager_internal(server)));
}

async fn wiki_manager_internal(server: &Server) {
    let wiki_id = server
        .create_wiki("Test Wiki", "test", "example.com")
        .await
        .expect("Unable to create wiki");

    server
        .rename_wiki(wiki_id, "NUTTEST")
        .await
        .expect("Unable to rename wiki");

    server
        .set_wiki_domain(wiki_id, "example.org")
        .await
        .expect("Unable to change domain");

    {
        let (wiki, _) = server
            .get_wiki_by_slug("test")
            .await
            .expect("Couldn't find wiki");

        assert_eq!(wiki_id, wiki.id());
    }

    {
        let error = server
            .get_wiki_by_slug("nonexistent")
            .await
            .expect_err("Found wiki");

        match error {
            Error::WikiNotFound => (),
            _ => panic!("Error doesn't match"),
        }
    }
}
