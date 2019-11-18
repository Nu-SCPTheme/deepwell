/*
 * test/page.rs
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
fn page_service() {
    run(|server| task::block_on(page_service_internal(server)));
}

async fn page_service_internal(srv: &Server) {
    let user = srv
        .get_user_from_name("unknown")
        .await
        .expect("Unable to get user")
        .expect("Default user not found");

    let wiki_id = srv
        .create_wiki("Test", "test", "example.org")
        .await
        .expect("Unable to create wiki");

    assert_eq!(srv.check_page(wiki_id, "tale-here").await.unwrap(), false);

    let commit = PageCommit {
        wiki_id,
        slug: &"tale-here",
        message: "new tale!",
        user: &user,
    };

    let (_page_id, _revision_id) = srv
        .create_page(commit, b"my great article here", &[], "Tale Thing", "")
        .await
        .expect("Unable to create page");

    assert_eq!(srv.check_page(wiki_id, "tale-here").await.unwrap(), true);

    srv.rename_page(
        wiki_id,
        "tale-here",
        "amazing-battle",
        "I like this name better",
        &user,
    )
    .await
    .expect("Unable to rename page");

    let commit = PageCommit {
        wiki_id,
        slug: &"amazing-battle",
        message: "changing title",
        user: &user,
    };

    srv.edit_page(
        commit,
        None,
        Some("Amazing Take-down of 682!"),
        Some("049 appears too"),
    )
    .await
    .expect("Unable to edit page");

    assert_eq!(srv.check_page(wiki_id, "tale-here").await.unwrap(), false);
    assert_eq!(
        srv.check_page(wiki_id, "amazing-battle").await.unwrap(),
        true
    );

    let commit = PageCommit {
        wiki_id,
        slug: &"amazing-battle",
        message: "people keep downvoting :(",
        user: &user,
    };

    srv.remove_page(commit)
        .await
        .expect("Unable to remove page");

    assert_eq!(srv.check_page(wiki_id, "nonexistent").await.unwrap(), false);
    assert_eq!(srv.check_page(wiki_id, "tale-here").await.unwrap(), false);
    assert_eq!(
        srv.check_page(wiki_id, "amazing-battle").await.unwrap(),
        false
    );
}
