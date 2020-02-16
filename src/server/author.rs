/*
 * server/author.rs
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

use super::utils::normalize_slug;
use crate::manager_prelude::*;
use crate::package::author::{Author, AuthorType};

impl Server {
    async fn get_page_id<S: Into<String>>(
        &self,
        page: Either<PageId, (WikiId, S)>,
    ) -> Result<PageId> {
        match page {
            Left(id) => Ok(id),
            Right((wiki_id, slug)) => {
                let slug = normalize_slug(slug);

                self.page
                    .get_page_id(wiki_id, &slug)
                    .await?
                    .ok_or(Error::PageNotFound)
            }
        }
    }

    /// Gets all authors for a given page.
    pub async fn get_page_authors(
        &self,
        page: Either<PageId, (WikiId, &str)>,
    ) -> Result<Vec<Author>> {
        info!("Getting authors for page {:?}", page);

        self.transaction(async {
            let page_id = self.get_page_id(page).await?;

            self.author.get_all(page_id).await
        })
        .await
    }

    /// Adds or sets a group of authors.
    pub async fn add_page_authors(
        &self,
        page: Either<PageId, (WikiId, &str)>,
        authors: &[(UserId, AuthorType, Option<NaiveDate>)],
    ) -> Result<()> {
        info!("Adding authors to page {:?}: {:?}", page, authors);

        self.transaction(async {
            let page_id = self.get_page_id(page).await?;

            for &(user_id, author_type, written_at) in authors {
                self.author
                    .add(page_id, user_id, author_type, written_at)
                    .await?;
            }

            Ok(())
        })
        .await
    }

    /// Removes a group of authors.
    pub async fn remove_page_authors(
        &self,
        page: Either<PageId, (WikiId, &str)>,
        authors: &[(UserId, AuthorType)],
    ) -> Result<usize> {
        info!("Removing authors from page {:?}: {:?}", page, authors);

        self.transaction(async {
            let page_id = self.get_page_id(page).await?;
            let mut count = 0;

            for (user_id, author_type) in authors.iter().copied() {
                let removed = self.author.remove(page_id, user_id, author_type).await?;
                if removed {
                    count += 1;
                }
            }

            Ok(count)
        })
        .await
    }
}
