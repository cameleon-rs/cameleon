/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub mod protocol;
pub mod register_map;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

pub const GVCP_DEFAULT_PORT: u16 = 3956;

use std::{net::Ipv4Addr, time};

use async_std::{future, net};
use tracing::warn;

use protocol::{ack, cmd, prelude::*};

#[tracing::instrument(level = "warn")]
pub async fn enumerate_devices(
    local_addr: Ipv4Addr,
    timeout: time::Duration,
) -> Result<Vec<ack::Discovery>> {
    let sock = net::UdpSocket::bind((local_addr, 0)).await?;
    let packet = cmd::Discovery::new().finalize(0xffff);
    let mut buf = [0_u8; 1024];
    packet.serialize(buf.as_mut()).unwrap();
    let length = packet.length() as usize;

    sock.set_broadcast(true)?;
    sock.send_to(&buf[..length], ("255.255.255.255", GVCP_DEFAULT_PORT))
        .await?;
    sock.set_broadcast(false).unwrap();

    let mut discoveries = vec![];

    while future::timeout(timeout, sock.recv_from(&mut buf))
        .await
        .is_ok()
    {
        if let Ok(ack) = ack::AckPacket::parse(&buf) {
            if ack.status().is_success() {
                match ack.ack_data_as::<ack::Discovery>() {
                    Ok(discovery) => discoveries.push(discovery),
                    Err(err) => warn!("{}", err),
                }
            } else {
                warn!("invalid discovery ack status: {:?}", ack.status())
            }
        }
    }

    Ok(discoveries)
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] async_std::io::Error),

    #[error("packet is broken: {0}")]
    InvalidPacket(std::borrow::Cow<'static, str>),

    #[error("invalid data: {0}")]
    InvalidData(std::borrow::Cow<'static, str>),
}
