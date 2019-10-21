/*
 * page/service.rs
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

use crate::revision::RevisionStore;
use crate::service_prelude::*;
use crate::wiki::WikiId;
use std::collections::HashMap;

pub struct PageService<'d> {
    conn: &'d PgConnection,
    stores: HashMap<WikiId, RevisionStore>,
}

impl<'d> PageService<'d> {
    #[inline]
    pub fn new(conn: &'d PgConnection) -> Self {
        PageService {
            conn,
            stores: HashMap::new(),
        }
    }

    /// Records the given page revision into the database and page store.
    pub fn commit<S, B>(&self, slug: S, content: B, wiki: WikiId, user: ()) -> Result<()> {
        unimplemented!()
    }
}
