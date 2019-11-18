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
    run(|handle| task::block_on(wiki_service_internal(handle)));
}

async fn wiki_service_internal(handle: &Handle) {
    let wiki_id = handle
        .create_wiki("Test Wiki", "test", "example.com")
        .await
        .expect("Unable to create wiki");

    handle.rename_wiki(wiki_id, "NUTTEST")
        .await
        .expect("Unable to rename wiki");

    handle.set_wiki_domain(wiki_id, "example.org")
        .await
        .expect("Unable to change domain");

    {
        let id = handle.get_wiki_id("test").await.expect("Couldn't find wiki");
        assert_eq!(id, wiki_id);
    }

    {
        let err = handle
            .get_wiki_id("nonexistent")
            .await
            .expect_err("Found wiki");
        match err {
            Error::WikiNotFound => (),
            _ => panic!("Error doesn't match"),
        }
    }
}
