/*
 * test/tags.rs
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
fn tags() {
    run(|server| task::block_on(tags_internal(server)));
}

async fn tags_internal(server: &Server) {
    let user_1 = server
        .get_user_from_name("unknown")
        .await
        .expect("Unable to get user")
        .expect("Default user not found");

    let user_2 = {
        let user_id = server
            .create_user("squirrelbird", "jenny@example.net", "blackmoonhowls")
            .await
            .expect("Unable to create user");

        server
            .get_user_from_id(user_id)
            .await
            .expect("Unable to get user")
    };

    let wiki_id = server
        .create_wiki("Test", "test", "example.org")
        .await
        .expect("Unable to create wiki");

    let commit = PageCommit {
        wiki_id,
        slug: "scp-xxxx",
        message: "New article!",
        user: &user_1,
    };

    let (_page_id, _revision_id) = server
        .create_page(
            commit,
            b"**Item #:** SCP-XXXX\n\n**Object Class:** Keter\n",
            &[],
            "SCP-XXXX",
            "The Monster Behind the Door",
        )
        .await
        .expect("Unable to create page");

    let commit = PageCommit {
        wiki_id,
        slug: "scp-xxxx",
        message: "has image",
        user: &user_1,
    };

    server
        .set_page_tags(commit, &["_image"])
        .await
        .expect("Unable to set page tags");

    let commit = PageCommit {
        wiki_id,
        slug: "scp-xxxx",
        message: "initial tagging",
        user: &user_2,
    };

    server
        .set_page_tags(
            commit,
            &["scp", "keter", "_image", "ontokinetic", "artifact"],
        )
        .await
        .expect("Unable to set page tags");

    let commit = PageCommit {
        wiki_id,
        slug: "scp-xxxx",
        message: "good image",
        user: &user_1,
    };

    server
        .set_page_tags(commit, &["scp", "keter", "artifact", "ontokinetic", "_cc"])
        .await
        .expect("Unable to set page tags");

    let commit = PageCommit {
        wiki_id,
        slug: "scp-xxxx",
        message: "goi tags",
        user: &user_2,
    };

    server
        .set_page_tags(
            commit,
            &[
                "scp",
                "keter",
                "artifact",
                "ontokinetic",
                "_cc",
                "chaos-insurgency",
                "ethics-committee",
            ],
        )
        .await
        .expect("Unable to set page tags");

    let (page, _) = server
        .get_page(wiki_id, "scp-xxxx")
        .await
        .expect("Unable to get page")
        .expect("No page found");

    let actual_tags = page
        .tags()
        .into_iter()
        .map(|tag| tag.as_str())
        .collect::<Vec<&str>>();

    let expected_tags = [
        "_cc",
        "artifact",
        "chaos-insurgency",
        "ethics-committee",
        "keter",
        "ontokinetic",
        "scp",
    ];

    assert_eq!(&actual_tags, &expected_tags);

    // Query by page tags

    let pages = server
        .get_pages_with_tags(wiki_id, &[])
        .await
        .expect("Unable to get pages from tags");

    assert!(pages.is_empty());

    let pages = server
        .get_pages_with_tags(wiki_id, &["keter"])
        .await
        .expect("Unable to get pages from tags");

    assert_eq!(pages.len(), 1);
    assert_eq!(pages[0].id(), page.id());

    let pages = server
        .get_pages_with_tags(wiki_id, &["keter", "ontokinetic"])
        .await
        .expect("Unable to get pages from tags");

    assert_eq!(pages.len(), 1);
    assert_eq!(pages[0].id(), page.id());

    let pages = server
        .get_pages_with_tags(wiki_id, &["ontokinetic", "keter"])
        .await
        .expect("Unable to get pages from tags");

    assert_eq!(pages.len(), 1);
    assert_eq!(pages[0].id(), page.id());

    let pages = server
        .get_pages_with_tags(wiki_id, &["esoteric-class", "ontokinetic"])
        .await
        .expect("Unable to get pages from tags");

    assert!(pages.is_empty());
}
