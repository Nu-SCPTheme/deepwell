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
    run(|handle| task::block_on(page_service_internal(handle)));
}

async fn page_service_internal(handle: &Handle) {
    // Setup
    let user = handle
        .get_user_from_name("unknown")
        .await
        .expect("Unable to get user")
        .expect("Default user not found");

    let wiki_id = handle
        .create_wiki("Test", "test", "example.org")
        .await
        .expect("Unable to create wiki");

    assert_eq!(
        handle.check_page(wiki_id, "tale-here").await.unwrap(),
        false
    );

    // Create page
    let commit = PageCommit {
        wiki_id,
        slug: &"tale-here",
        message: "new tale!",
        user: &user,
    };

    let (page_id, _revision_id) = handle
        .create_page(commit, b"my great article here", &[], "Tale Thing", "")
        .await
        .expect("Unable to create page");

    assert_eq!(handle.check_page(wiki_id, "tale-here").await.unwrap(), true);

    // Rename and edits
    handle
        .rename_page(
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

    handle
        .edit_page(
            commit,
            None,
            Some("Amazing Take-down of 682!"),
            Some("049 appears too"),
        )
        .await
        .expect("Unable to edit page");

    assert_eq!(
        handle.check_page(wiki_id, "tale-here").await.unwrap(),
        false
    );
    assert_eq!(
        handle.check_page(wiki_id, "amazing-battle").await.unwrap(),
        true
    );

    let commit = PageCommit {
        wiki_id,
        slug: &"amazing-battle",
        message: "rewrite main battle",
        user: &user,
    };

    let revision_id = handle
        .edit_page(
            commit,
            Some(b"and then 049 cured him!! it was epic"),
            None,
            None,
        )
        .await
        .expect("Unable to edit page");

    let commit = PageCommit {
        wiki_id,
        slug: &"amazing-battle",
        message: "switching back to previous story",
        user: &user,
    };

    handle
        .undo_revision(commit, Left(revision_id))
        .await
        .expect("Unable to undo page revision");

    // Remove page
    let commit = PageCommit {
        wiki_id,
        slug: &"amazing-battle",
        message: "people keep downvoting :(",
        user: &user,
    };

    handle
        .remove_page(commit)
        .await
        .expect("Unable to remove page");

    assert_eq!(
        handle.check_page(wiki_id, "nonexistent").await.unwrap(),
        false
    );
    assert_eq!(
        handle.check_page(wiki_id, "tale-here").await.unwrap(),
        false
    );
    assert_eq!(
        handle.check_page(wiki_id, "amazing-battle").await.unwrap(),
        false
    );

    let commit = PageCommit {
        wiki_id,
        slug: &"amazing-battle",
        message: "Un-delete by moderator order",
        user: &user,
    };

    // Restore page
    handle
        .restore_page(commit, Some(page_id))
        .await
        .expect("Unable to restore page");

    assert_eq!(
        handle.check_page(wiki_id, "tale-here").await.unwrap(),
        false
    );
    assert_eq!(
        handle.check_page(wiki_id, "amazing-battle").await.unwrap(),
        true
    );
}
