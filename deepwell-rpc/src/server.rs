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
use crate::async_deepwell::*;
use crate::Result;
use async_std::task;
use deepwell::{Config as DeepwellConfig, Server as DeepwellServer};
use futures::future::{self, Ready};
use futures::prelude::*;
use futures::channel::mpsc;
use ipnetwork::IpNetwork;
use std::io;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::SystemTime;
use tarpc::context::Context;
use tarpc::serde_transport::tcp;
use tarpc::server::{BaseChannel, Channel};
use tokio_serde::formats::Json;

// Prevent network socket exhaustion or related slowdown
const MAX_PARALLEL_REQUESTS: usize = 16;

// Applies backpressure to AsyncDeepwell
const QUEUE_SIZE: usize = 256;

#[derive(Debug, Clone)]
pub struct Server {
    channel: mpsc::Sender<AsyncDeepwellRequest>,
}

impl Server {
    #[inline]
    pub fn init(config: DeepwellConfig) -> Self {
        info!("Initializing DEEPWELL server");
        let (send, recv) = mpsc::channel(QUEUE_SIZE);

        task::spawn(async move {
            let server = DeepwellServer::new(config)
                .expect("Unable to start DEEPWELL server");

            AsyncDeepwell::new(server, recv).run().await;
        });

        Self { channel: send }
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
    type LoginFut = Ready<Result<()>>;

    fn login(
        self,
        _: Context,
        username_or_email: String,
        password: String,
        ip_address: IpAddr,
    ) -> Self::LoginFut {
        info!("Method: login");

        let network = get_network(ip_address);

        let fut = self.inner.try_login(&username_or_email, &password, network);
        unimplemented!()
        //let result = fut.await.map_err(|error| SendableError::from(error));
    }

    // TODO
}

fn get_network(ip: IpAddr) -> IpNetwork {
    use ipnetwork::{Ipv4Network, Ipv6Network};
    use std::net::{Ipv4Addr, Ipv6Addr};

    fn convert_v4(ip: Ipv4Addr) -> Ipv4Network {
        Ipv4Network::new(ip, 32).expect("Unable to convert IPv4 address")
    }

    fn convert_v6(ip: Ipv6Addr) -> Ipv6Network {
        Ipv6Network::new(ip, 128).expect("Unable to convert IPv6 address")
    }

    match ip {
        IpAddr::V4(ip) => IpNetwork::V4(convert_v4(ip)),
        IpAddr::V6(ip) => IpNetwork::V6(convert_v6(ip)),
    }
}
