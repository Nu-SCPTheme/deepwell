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
    run(|srv| {
        let user = srv
            .get_user_from_name("unknown")
            .expect("Unable to get user")
            .expect("Default user not found");

        let wiki_id = srv
            .create_wiki("Test", "test", "example.org")
            .expect("Unable to create wiki");

        assert_eq!(srv.check_page(wiki_id, "tale-here").unwrap(), false);

        let (_page_id, _revision_id) = srv
            .create_page(
                wiki_id,
                "tale-here",
                b"my great article here",
                "new s&c plastics",
                &user,
                &[],
                "Tale Thing",
                "",
            )
            .expect("Unable to create page");

        assert_eq!(srv.check_page(wiki_id, "tale-here").unwrap(), true);

        srv.rename_page(
            wiki_id,
            "tale-here",
            "amazing-battle",
            "I like this name better",
            &user,
        )
        .expect("Unable to rename page");

        srv.edit_page(
            wiki_id,
            "amazing-battle",
            None,
            "changing title",
            &user,
            Some("Amazing Take-down of 682!"),
            Some("049 appears too"),
        )
        .expect("Unable to edit page");

        assert_eq!(srv.check_page(wiki_id, "tale-here").unwrap(), false);
        assert_eq!(srv.check_page(wiki_id, "amazing-battle").unwrap(), true);

        srv.remove_page(
            wiki_id,
            "amazing-battle",
            "people keep downvoting :(",
            &user,
        )
        .expect("Unable to remove page");

        assert_eq!(srv.check_page(wiki_id, "nonexistent").unwrap(), false);
        assert_eq!(srv.check_page(wiki_id, "tale-here").unwrap(), false);
        assert_eq!(srv.check_page(wiki_id, "amazing-battle").unwrap(), false);
    });
}
