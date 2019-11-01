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
    run(|srv| {
        let user_1 = srv
            .get_user_from_name("unknown")
            .expect("Unable to get user")
            .expect("Default user not found");

        let user_2 = {
            let user_id = srv
                .create_user("squirrelbird", "jenny@example.net", "blackmoonhowls")
                .expect("Unable to create user");

            srv.get_user_from_id(user_id).expect("Unable to get user")
        };

        let wiki_id = srv
            .create_wiki("Test", "test", "example.org")
            .expect("Unable to create wiki");

        let (_page_id, _revision_id) = srv
            .create_page(
                wiki_id,
                "scp-xxxx",
                b"**Item #:** SCP-XXXX\n\n**Object Class:** Keter\n",
                "New article!",
                &user_1,
                &[],
                "SCP-XXXX",
                "The Monster Behind the Door",
            )
            .expect("Unable to create page");

        srv.set_page_tags(wiki_id, "scp-xxxx", "has image", &user_1, &["_image"])
            .expect("Unable to set page tags");

        srv.set_page_tags(
            wiki_id,
            "scp-xxxx",
            "initial tagging",
            &user_2,
            &["scp", "keter", "_image", "ontokinetic", "artifact"],
        )
        .expect("Unable to set page tags");

        srv.set_page_tags(
            wiki_id,
            "scp-xxxx",
            "good image",
            &user_1,
            &["scp", "keter", "artifact", "ontokinetic", "_cc"],
        )
        .expect("Unable to set page tags");

        srv.set_page_tags(
            wiki_id,
            "scp-xxxx",
            "goi tags",
            &user_2,
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
        .expect("Unable to set page tags");

        let (page, _) = srv
            .get_page(wiki_id, "scp-xxxx")
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
    });
}
