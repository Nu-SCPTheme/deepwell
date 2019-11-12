/*
 * server/wiki.rs
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

use super::utils::{normalize_slug, to_lowercase};
use crate::service_prelude::*;
use crate::wiki::UpdateWiki;

impl Server {
    /// Creates a new wiki with the given parameters. Returns its ID.
    pub fn create_wiki<S1, S2>(&self, name: &str, slug: S1, domain: S2) -> Result<WikiId>
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        let slug = normalize_slug(slug);
        let domain = to_lowercase(domain);

        task::block_on(async {
            let id = self.wiki.create(name, &slug, &domain).await?;

            self.wiki
                .get_by_id(id, |wiki| {
                    let wiki = wiki.expect("Can't find wiki object after inserting");

                    self.page.add_store(&wiki)?;
                    Ok(id)
                })
                .await
        })
    }

    /// Renames the given wiki.
    /// Changing a wiki's slug is not supported.
    pub fn rename_wiki(&self, id: WikiId, new_name: &str) -> Result<()> {
        let model = UpdateWiki {
            name: Some(new_name),
            domain: None,
        };

        info!("Renaming wiki ID {} to '{}'", id, new_name);

        self.wiki.edit(id, model)?;
        Ok(())
    }

    /// Changes the associated domain for the given wiki.
    pub fn set_wiki_domain(&self, id: WikiId, new_domain: &str) -> Result<()> {
        let model = UpdateWiki {
            name: None,
            domain: Some(new_domain),
        };

        info!("Changing domain for wiki ID {} to '{}'", id, new_domain);

        self.conn.transaction::<_, Error, _>(|| {
            self.wiki.edit(id, model)?;
            self.page.set_domain(id, new_domain)?;

            Ok(())
        })
    }

    /// Gets the wiki ID with the given slug.
    /// Returns an error if the wiki doesn't exist.
    pub fn get_wiki_id<S: Into<String>>(&self, slug: S) -> Result<WikiId> {
        task::block_on(async {
            let slug = normalize_slug(slug);

            self.wiki
                .get_by_slug(&slug, |wiki| match wiki {
                    Some(wiki) => Ok(wiki.id()),
                    None => Err(Error::WikiNotFound),
                })
                .await
        })
    }
}
