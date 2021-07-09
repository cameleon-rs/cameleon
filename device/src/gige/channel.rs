/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::net::SocketAddrV4;

use async_std::net::UdpSocket;

use super::Result;

pub struct ControlChannel {
    sock: UdpSocket,
    is_opened: bool,
}

impl ControlChannel {
    pub async fn send(&mut self, buf: &[u8]) -> Result<usize> {
        self.sock.send(buf).await.map_err(Into::into)
    }

    pub async fn recv(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.sock.recv(buf).await.map_err(Into::into)
    }

    pub async fn bind(&mut self, local_addr: SocketAddrV4) -> Result<()> {
        let dest_addr = self.sock.peer_addr()?;
        self.sock = UdpSocket::bind(local_addr).await?;
        self.sock.connect(dest_addr).await?;
        Ok(())
    }
}
