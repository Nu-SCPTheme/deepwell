/*
 * test/lock.rs
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
fn locks() {
    run(|server| task::block_on(locks_internal(server)));
}

async fn locks_internal(server: &Server) {
    // Setup models
    let wiki_id = create_wiki(server).await;
    let user_id = create_user(server).await;
    let other_user_id = create_user(server).await;

    let user = server
        .get_user_from_id(user_id)
        .await
        .expect("Unable to get user")
        .expect("Created user not found");

    let other_user = server
        .get_user_from_id(other_user_id)
        .await
        .expect("Unable to get user")
        .expect("Created user not found");

    // Setup state
    let mut commit = PageCommit {
        wiki_id,
        slug: &"tale-here",
        message: "New tale!",
        user: &user,
    };

    macro_rules! check_err {
        ($error:expr, $user_id:expr) => {
            match $error {
                Error::PageLocked(id) if id == $user_id => (),
                Error::PageLocked(_) => panic!("Page locked with unknown user"),
                _ => panic!("Error doesn't match"),
            }
        };
    }

    macro_rules! edit {
        ($content:expr) => {
            server
                .edit_page(commit, Some($content), None, None)
                .await
                .expect("Unable to edit page")
        };
        ($content:expr, $user_id:expr) => {{
            let error = server
                .edit_page(commit, Some($content), None, None)
                .await
                .expect_err("Able to edit page despite lock");

            check_err!(error, $user_id);
        }};
    }

    // Create page
    let content = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit.";
    server
        .create_page(commit, content, &[], "The Grand Story of Dude", "")
        .await
        .expect("Unable to create page");

    // Create page lock
    // All operations done by other_user_id should fail
    server
        .create_page_lock(wiki_id, "tale-here", user_id)
        .await
        .expect("Unable to create page lock");

    // Edit page with no contention
    commit.message = "Updating tale";
    edit!(b"Proin est lectus, venenatis nec convallis a, rhoncus nec lorem.");

    // Edit page with lock
    commit.message = "More updates";
    edit!(b"Donec vel augue id massa semper vehicula iaculis ac sapien.");

    // Edit page with no lock (failure)
    commit.message = "Some other user's changes";
    commit.user = &other_user;
    edit!(b"Apple banana cherry durian", user_id);

    // Remove page lock
    server
        .remove_page_lock(wiki_id, "tale-here")
        .await
        .expect("Unable to remove page lock");

    // Edit page with no lock (success)
    commit.message = "Some other user's changes";
    commit.user = &other_user;
    edit!(b"Apple banana cherry durian");

    // Create page lock
    // All operations done by user_id should fail
    server
        .create_page_lock(wiki_id, "tale-here", other_user_id)
        .await
        .expect("Unable to create page lock");

    // Rename page (failure)
    let error = server
        .rename_page(
            wiki_id,
            "tale-here",
            "troll-lol",
            "Try to rename page lol",
            &user,
        )
        .await
        .expect_err("Renaming page succeeded despite lock");

    check_err!(error, other_user_id);

    // Tag page (failure)
    commit.message = "Adding tags";
    commit.user = &user;

    let error = server
        .set_page_tags(commit, &["badass", "scp-049", "powerful"])
        .await
        .expect_err("Allowed to set tags despite lock");

    check_err!(error, other_user_id);

    // Remove page (failure)
    commit.message = "Deleting tale";
    commit.user = &user;

    let error = server
        .remove_page(commit)
        .await
        .expect_err("Unable to remove page");

    check_err!(error, other_user_id);

    // Rename page (success)
    server
        .rename_page(
            wiki_id,
            "tale-here",
            "my-tale-here",
            "Renaming page",
            &other_user,
        )
        .await
        .expect("Unable to rename page");

    server
        .rename_page(
            wiki_id,
            "my-tale-here",
            "tale-here",
            "Nevermind",
            &other_user,
        )
        .await
        .expect("Unable to rename page");

    // Tag page (success)
    commit.message = "Adding tags";
    commit.user = &other_user;

    server
        .set_page_tags(commit, &["tale", "herman-fuller", "_image"])
        .await
        .expect("Unable to set tags");

    // Remove page (success)
    commit.message = "Deleting tale";
    commit.user = &other_user;

    server
        .remove_page(commit)
        .await
        .expect("Unable to remove page");

    // Remove page lock
    server
        .remove_page_lock(wiki_id, "tale-here")
        .await
        .expect("Unable to remove page lock");
}
