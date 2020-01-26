/*
 * deepwell.rs
 *
 * deepwell-rpc - RPC server to provide database management and migrations
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

//! Helper struct to keep `deepwell::Server` in a fixed memory position,
//! and use `Send + Sync` future channels to communicate with it.

use deepwell::Server as DeepwellServer;
use futures::channel::{mpsc, oneshot};
use futures::prelude::*;

#[derive(Debug)]
pub struct AsyncDeepwell {
    server: DeepwellServer,
    channel: mpsc::Receiver<AsyncDeepwellRequest>,
}

impl AsyncDeepwell {
    #[inline]
    pub fn new(server: DeepwellServer, channel: mpsc::Receiver<AsyncDeepwellRequest>) -> Self {
        Self { server, channel }
    }

    #[inline]
    pub fn sender(&self) -> mpsc::Sender<AsyncDeepwellRequest> {
        mpsc::Sender::clone(&self.send)
    }

    pub async fn run(&mut self) {
        while let Some(request) = self.channel.next().await {
            match request {
                // TODO
            }
        }

        panic!("Receiver stream exhausted");
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AsyncDeepwellRequest {
}
