/*
 * server.rs
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

use crate::api::{Deepwell as DeepwellApi, PROTOCOL_VERSION};
use crate::async_deepwell::AsyncDeepwellRequest;
use crate::{Result, StdResult};
use deepwell_core::Error as DeepwellError;
use futures::channel::{mpsc, oneshot};
use futures::future::{self, BoxFuture, Ready};
use futures::prelude::*;
use std::io;
use std::net::SocketAddr;
use std::time::SystemTime;
use tarpc::context::Context;
use tarpc::serde_transport::tcp;
use tarpc::server::{BaseChannel, Channel};
use tokio_serde::formats::Json;

// Prevent network socket exhaustion or related slowdown
const MAX_PARALLEL_REQUESTS: usize = 16;

#[derive(Debug, Clone)]
pub struct Server {
    channel: mpsc::Sender<AsyncDeepwellRequest>,
}

impl Server {
    #[inline]
    pub fn init(channel: mpsc::Sender<AsyncDeepwellRequest>) -> Self {
        Self { channel }
    }

    pub async fn run(&self, address: SocketAddr) -> io::Result<()> {
        tcp::listen(&address, Json::default)
            .await?
            // Log requests
            .filter_map(|conn| {
                async move {
                    match conn {
                        // Note incoming connection
                        Ok(conn) => {
                            match conn.peer_addr() {
                                Ok(addr) => info!("Accepted connection from {}", addr),
                                Err(error) => warn!("Unable to get peer address: {}", error),
                            }

                            Some(conn)
                        }
                        // Unable to accept connection
                        Err(error) => {
                            warn!("Error accepting connection: {}", error);

                            None
                        }
                    }
                }
            })
            // Create and fulfill channels for each request
            .map(BaseChannel::with_defaults)
            .map(|chan| {
                let resp = self.clone().serve();
                chan.respond_with(resp).execute()
            })
            .buffer_unordered(MAX_PARALLEL_REQUESTS)
            .for_each(|_| async {})
            .await;

        Ok(())
    }

    /// Enqueues the deepwell request on the mpsc, and awaits the result oneshot for the result.
    async fn call<T>(
        &mut self,
        request: AsyncDeepwellRequest,
        recv: oneshot::Receiver<StdResult<T, DeepwellError>>,
    ) -> Result<T> {
        self.channel
            .send(request)
            .await
            .expect("Deepwell server channel closed");

        recv.await
            .expect("Oneshot closed before result")
            .map_err(|e| e.to_sendable())
    }
}

impl DeepwellApi for Server {
    // Misc

    type ProtocolFut = Ready<String>;

    #[inline]
    fn protocol(self, _: Context) -> Self::ProtocolFut {
        info!("Method: protocol");

        future::ready(str!(PROTOCOL_VERSION))
    }

    type PingFut = Ready<String>;

    #[inline]
    fn ping(self, _: Context) -> Self::PingFut {
        info!("Method: ping");

        future::ready(str!("pong!"))
    }

    type TimeFut = Ready<f64>;

    #[inline]
    fn time(self, _: Context) -> Self::TimeFut {
        info!("Method: time");

        let now = SystemTime::now();
        let unix_time = now
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("System time before epoch")
            .as_secs_f64();

        future::ready(unix_time)
    }

    // Sessions
    type LoginFut = BoxFuture<'static, Result<()>>;

    fn login(
        mut self,
        _: Context,
        username_or_email: String,
        password: String,
        remote_address: Option<String>,
    ) -> Self::LoginFut {
        info!("Method: login");

        let fut = async move {
            let (send, recv) = oneshot::channel();

            let request = AsyncDeepwellRequest::TryLogin {
                username_or_email,
                password,
                remote_address,
                response: send,
            };

            self.call(request, recv).await
        };

        fut.boxed()
    }

    // TODO
}
