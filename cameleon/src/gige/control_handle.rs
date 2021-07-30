/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::{convert::TryInto, time};

use async_std::{future, net::UdpSocket, task};

use crate::{ControlError, ControlResult, DeviceControl};

use cameleon_device::gige::protocol::{ack, cmd};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenMode {
    Exclusive,
    Control,
    MonitorAccess,
}

#[derive(Debug)]
pub struct ControlHandleInner {
    sock: UdpSocket,
    is_opened: bool,
    config: ConnectionConfig,
    next_req_id: u16,
    buf: Vec<u8>,
}

macro_rules! align {
    ($data:expr) => {
        ($data + 1) & !0x11
    };
}

impl ControlHandleInner {
    /// Sets [`OpenMode`] of the handle. Default is [`OpenMode::Exclusive`].
    pub fn set_open_mode(&mut self, mode: OpenMode) {
        self.config.open_mode = mode;
    }

    pub fn open_mode(&self) -> OpenMode {
        self.config.open_mode
    }

    /// Sets timeout duration of each transaction.
    pub fn set_timeout(&mut self, timeout: time::Duration) {
        self.config.timeout = timeout;
    }

    pub fn timeout(&self) -> time::Duration {
        self.config.timeout
    }

    fn assert_open(&self) -> ControlResult<()> {
        if self.is_opened() {
            Ok(())
        } else {
            Err(ControlError::NotOpened)
        }
    }

    async fn send_cmd<'a, T, U>(&'a mut self, cmd: T) -> ControlResult<U>
    where
        T: cmd::CommandData,
        U: ack::ParseAckData<'a>,
    {
        unwrap_or_log!(self.assert_open());
        let cmd = cmd.finalize(self.next_req_id);
        let cmd_len = cmd.length() as usize;
        cmd.serialize(self.buf.as_mut_slice())?;

        self.send(cmd_len).await?;

        self.recv().await?;
        let mut retry_count = self.config.retry_count;

        loop {
            self.recv().await?;
            let ack = ack::AckPacket::parse(&self.buf)?;
            self.verify_ack(&ack)?;

            if ack.ack_kind() == ack::AckKind::Pending {
                let pending_ack: ack::Pending = ack.ack_data_as()?;
                let waiting_time = pending_ack.waiting_time();
                task::sleep(waiting_time).await;
                retry_count -= 1;
                if retry_count == 0 {
                    return Err(ControlError::Io(anyhow::Error::msg(
                        "the number of times pending was returned exceeds the retry_count.",
                    )));
                }
                continue;
            }

            self.next_req_id += 1;
            break;
        }

        ack::AckPacket::parse(&self.buf)?
            .ack_data_as()
            .map_err(Into::into)
    }

    fn verify_ack(&self, ack: &ack::AckPacket) -> ControlResult<()> {
        let status = ack.status();
        if !status.is_success() {
            return Err(ControlError::Io(anyhow::Error::msg(format!(
                "invalid status: {:?}",
                ack.status().kind()
            ))));
        }

        if ack.request_id() != self.next_req_id {
            return Err(ControlError::Io(anyhow::Error::msg("request id mismatch")));
        }

        Ok(())
    }

    pub async fn send(&self, len: usize) -> ControlResult<usize> {
        timeout(self.timeout(), self.sock.send(&self.buf[..len]))
            .await?
            .map_err(Into::into)
    }

    pub async fn recv(&mut self) -> ControlResult<usize> {
        timeout(self.timeout(), self.sock.recv(&mut self.buf))
            .await?
            .map_err(Into::into)
    }
}

impl DeviceControl for ControlHandleInner {
    fn open(&mut self) -> ControlResult<()> {
        match self.config.open_mode {
            OpenMode::Exclusive => todo!(),
            OpenMode::Control => todo!(),
            OpenMode::MonitorAccess => Ok(()),
        }
    }

    fn close(&mut self) -> ControlResult<()> {
        todo!()
    }

    fn is_opened(&self) -> bool {
        self.is_opened
    }

    fn read(&mut self, mut address: u64, buf: &mut [u8]) -> ControlResult<()> {
        debug_assert_eq!(cmd::ReadMem::maximum_read_length() % 4, 0);

        for buf_chunk in buf.chunks_mut(cmd::ReadMem::maximum_read_length()) {
            let target_addr: u32 = address.try_into().map_err(|_| {
                ControlError::InvalidData(
                    "the address of `ReadMem` command must be smaller than u32::MAX".into(),
                )
            })?;
            let read_len = buf_chunk.len() as u16;
            let aligned_read_len = align!(read_len);

            let cmd = unwrap_or_log!(cmd::ReadMem::new(target_addr, aligned_read_len));
            let ack: ack::ReadMem = unwrap_or_log!(task::block_on(self.send_cmd(cmd)));
            buf_chunk.copy_from_slice(&ack.data()[..read_len as usize]);

            address += read_len as u64;
        }

        Ok(())
    }

    fn read_reg(&mut self, address: u64) -> ControlResult<[u8; 4]> {
        let address: u32 = address.try_into().map_err(|_| {
            ControlError::InvalidData(
                "the address of `ReadReg` command must be smaller than u32::MAX".into(),
            )
        })?;

        let mut cmd = cmd::ReadReg::new();
        unwrap_or_log!(cmd.add_entry(address));
        let ack: ack::ReadReg = unwrap_or_log!(task::block_on(self.send_cmd(cmd)));
        Ok(unwrap_or_log!(ack
            .iter()
            .next()
            .ok_or_else(|| {
                ControlError::Io(anyhow::Error::msg("no entry in `ReadReg` ack packet"))
            })
            .map(|v| *v)))
    }

    fn write(&mut self, mut address: u64, data: &[u8]) -> ControlResult<()> {
        debug_assert_eq!(cmd::WriteMem::maximum_data_length() % 4, 0);

        for data_chunk in data.chunks(cmd::WriteMem::maximum_data_length()) {
            let target_addr: u32 = address.try_into().map_err(|_| {
                ControlError::InvalidData(
                    "the address of `ReadMem` command must be smaller than u32::MAX".into(),
                )
            })?;
            let aligned_data_len = align!(data_chunk.len());

            let _: ack::WriteMem = if aligned_data_len == data_chunk.len() {
                let cmd = unwrap_or_log!(cmd::WriteMem::new(target_addr, data_chunk));
                unwrap_or_log!(task::block_on(self.send_cmd(cmd)))
            } else {
                let mut aligned_data = vec![0; aligned_data_len];
                aligned_data[..data_chunk.len()].copy_from_slice(data_chunk);
                let cmd = unwrap_or_log!(cmd::WriteMem::new(target_addr, data_chunk));
                unwrap_or_log!(task::block_on(self.send_cmd(cmd)))
            };

            address += aligned_data_len as u64;
        }

        Ok(())
    }

    fn write_reg(&mut self, address: u64, data: [u8; 4]) -> ControlResult<()> {
        let address: u32 = address.try_into().map_err(|_| {
            ControlError::InvalidData(
                "the address of `ReadReg` command must be smaller than u32::MAX".into(),
            )
        })?;

        let mut cmd = cmd::WriteReg::new();
        unwrap_or_log!(cmd.add_entry(unwrap_or_log!(cmd::WriteRegEntry::new(address, data))));
        let ack: ack::WriteReg = unwrap_or_log!(task::block_on(self.send_cmd(cmd)));

        if ack.entry_num() == 1 {
            Ok(())
        } else {
            unwrap_or_log!(Err(ControlError::Io(anyhow::Error::msg(
                "`WriteReg` failed: written entry num mismatch"
            ))))
        }
    }

    fn genapi(&mut self) -> ControlResult<String> {
        todo!()
    }

    fn enable_streaming(&mut self) -> ControlResult<()> {
        todo!()
    }

    fn disable_streaming(&mut self) -> ControlResult<()> {
        todo!()
    }
}

#[derive(Debug, Clone)]
struct ConnectionConfig {
    /// [`OpenMode`] of the handle. Default is [`OpenMode::Exclusive`].
    open_mode: OpenMode,

    /// Timeout duration of each transaction.
    timeout: time::Duration,

    /// The value determines how many times to retry when pending acknowledge is returned from the
    /// device.
    retry_count: u32,
}

impl From<async_std::io::Error> for ControlError {
    fn from(err: async_std::io::Error) -> Self {
        ControlError::Io(err.into())
    }
}

async fn timeout<F, T>(timeout: time::Duration, f: F) -> ControlResult<T>
where
    F: std::future::Future<Output = T>,
{
    future::timeout(timeout, f)
        .await
        .map_err(|_| ControlError::Timeout)
}
