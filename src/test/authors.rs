/*
 * test/authors.rs
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
use crate::author::AuthorType;

#[test]
fn author_service() {
    run(|server| task::block_on(author_service_impl(server)));
}

async fn author_service_impl(srv: &Server) {
    let wiki_id = srv
        .create_wiki("Test", "test", "example.org")
        .await
        .expect("Unable to create wiki");

    let user_1 = {
        let user_id = srv
            .create_user("superpersonyeah", "ralph@example.net", "blackmoonhowls")
            .await
            .expect("Unable to create user");

        srv.get_user_from_id(user_id)
            .await
            .expect("Unable to get user")
    };

    let user_2 = {
        let user_id = srv
            .create_user(
                "so many forgotten accounts",
                "smfa@example.net",
                "ribbon-person",
            )
            .await
            .expect("Unable to create user");

        srv.get_user_from_id(user_id)
            .await
            .expect("Unable to get user")
    };

    let user_3 = srv
        .get_user_from_name("unknown")
        .await
        .expect("Unable to get user")
        .expect("Default user not found");

    let commit = PageCommit {
        wiki_id,
        slug: "scp-xxxx",
        message: "new scp!!",
        user: &user_1,
    };

    let (page_id, _revision_id) = srv
        .create_page(
            commit,
            b"item number spc-xxx\nobject: SUPER KETER",
            &[],
            "SCP-XXXX",
            "Super-Keter",
        )
        .await
        .expect("Unable to create page");

    let page = Left(page_id);
    let authors = srv
        .get_page_authors(page)
        .await
        .expect("Unable to get page authors");

    assert_eq!(authors.len(), 1);

    assert_eq!(authors[0].user_id(), user_1.id());
    assert_eq!(authors[0].page_id(), page_id);
    assert_eq!(authors[0].author_type(), AuthorType::Author);

    srv.add_page_authors(
        page,
        &[
            (user_1.id(), AuthorType::Rewrite, None),
            (user_2.id(), AuthorType::Translator, None),
            (user_3.id(), AuthorType::Author, None),
        ],
    )
    .await
    .expect("Unable to add authors");

    let authors = srv
        .get_page_authors(page)
        .await
        .expect("Unable to get page authors");

    assert_eq!(authors.len(), 4);

    assert_eq!(authors[0].user_id(), user_3.id());
    assert_eq!(authors[0].page_id(), page_id);
    assert_eq!(authors[0].author_type(), AuthorType::Author);

    assert_eq!(authors[1].user_id(), user_1.id());
    assert_eq!(authors[1].page_id(), page_id);
    assert_eq!(authors[1].author_type(), AuthorType::Author);

    assert_eq!(authors[2].user_id(), user_1.id());
    assert_eq!(authors[2].page_id(), page_id);
    assert_eq!(authors[2].author_type(), AuthorType::Rewrite);

    assert_eq!(authors[3].user_id(), user_2.id());
    assert_eq!(authors[3].page_id(), page_id);
    assert_eq!(authors[3].author_type(), AuthorType::Translator);
}
