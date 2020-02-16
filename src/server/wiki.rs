/*
 * server/wiki.rs
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

use super::utils::{normalize_slug, to_lowercase};
use crate::manager_prelude::*;

impl Server {
    /// Creates a new wiki with the given parameters. Returns its ID.
    pub async fn create_wiki<S1, S2>(&self, name: &str, slug: S1, domain: S2) -> Result<WikiId>
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        let slug = normalize_slug(slug);
        let domain = to_lowercase(domain);

        let (id, guard) = self.wiki.create(name, &slug, &domain).await?;
        let wiki = guard
            .get(&id)
            .expect("Can't find wiki object after inserting");

        self.page.add_store(&wiki).await?;

        Ok(id)
    }

    /// Renames the given wiki.
    /// Changing a wiki's slug is not supported.
    pub async fn rename_wiki(&self, id: WikiId, new_name: &str) -> Result<()> {
        info!("Renaming wiki ID {} to '{}'", id, new_name);

        self.wiki.edit(id, Some(new_name), None).await?;
        Ok(())
    }

    /// Changes the associated domain for the given wiki.
    pub async fn set_wiki_domain(&self, id: WikiId, new_domain: &str) -> Result<()> {
        info!("Changing domain for wiki ID {} to '{}'", id, new_domain);

        self.transaction(async {
            self.wiki.edit(id, None, Some(new_domain)).await?;
            self.page.set_domain(id, new_domain).await?;

            Ok(())
        })
        .await
    }

    /// Gets information about the wiki with the given ID
    #[inline]
    pub async fn get_wiki_by_id(&self, id: WikiId) -> Result<(Wiki, WikiSettings)> {
        try_join!(self.wiki.get_by_id(id), self.wiki.get_settings(id))
    }

    /// Gets the wiki ID with the given slug.
    /// Returns an error if the wiki doesn't exist.
    pub async fn get_wiki_by_slug<S: Into<String>>(&self, slug: S) -> Result<(Wiki, WikiSettings)> {
        let slug = normalize_slug(slug);

        let wiki = self.wiki.get_by_slug(&slug).await?;
        let settings = self.wiki.get_settings(wiki.id()).await?;

        Ok((wiki, settings))
    }

    /// Edits the settings for the given wiki.
    pub async fn edit_wiki_settings(
        &self,
        id: WikiId,
        page_lock_duration: Option<i16>,
    ) -> Result<()> {
        info!(
            "Changing settings for wiki ID {}: page_lock_duration {:?}",
            id, page_lock_duration,
        );

        self.wiki.edit_settings(id, page_lock_duration).await
    }
}
