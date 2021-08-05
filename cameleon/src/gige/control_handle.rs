/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::{convert::TryInto, io::Read, time};

use async_std::{future, net::UdpSocket, task};

use cameleon_device::gige::protocol::{ack, cmd};

use crate::{
    genapi::CompressionType, utils::unzip_genxml, ControlError, ControlResult, DeviceControl,
};

use super::register_map::{Bootstrap, ControlChannelPriviledge, GvcpCapability, XmlFileLocation};

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
    buffer: Vec<u8>,
    capability: Option<GvcpCapability>,
}

macro_rules! align {
    ($data:expr) => {
        ($data + 3) & !0b11
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

    /// Capacity of the buffer, the buffer is used for
    /// serializing/deserializing packet. This buffer automatically extend according to packet
    /// length.
    pub fn buffer_capacity(&self) -> usize {
        self.buffer.capacity()
    }

    /// Resize the capacity of the buffer, the buffer is used for
    /// serializing/deserializing packet. This buffer automatically extend according to packet
    /// length.
    pub fn resize_buffer(&mut self, size: usize) {
        self.buffer.resize(size, 0);
        self.buffer.shrink_to_fit();
    }

    fn assert_open(&self) -> ControlResult<()> {
        if self.is_opened() {
            Ok(())
        } else {
            Err(ControlError::NotOpened)
        }
    }

    fn read_reg_fallback(&mut self, mut address: u64, buf: &mut [u8]) -> ControlResult<()> {
        for buf_chunk in buf.chunks_mut(4) {
            let data = self.read_reg(address)?;
            let chunk_len = buf_chunk.len();
            buf_chunk.copy_from_slice(&data[..chunk_len]);
            address += chunk_len as u64;
        }

        Ok(())
    }

    fn write_reg_fallback(&mut self, mut address: u64, data: &[u8]) -> ControlResult<()> {
        for data_chunk in data.chunks(4) {
            let chunk_len = data_chunk.len();
            if chunk_len == 4 {
                self.write_reg(address, data_chunk.try_into().unwrap())?;
            } else {
                let mut aligned_data = [0; 4];
                aligned_data[..chunk_len].copy_from_slice(data_chunk);
                self.write_reg(address, aligned_data)?;
            }
            address += chunk_len as u64;
        }

        Ok(())
    }

    async fn send_cmd<'a, T, U>(&'a mut self, cmd: T) -> ControlResult<U>
    where
        T: cmd::CommandData,
        U: ack::ParseAckData<'a>,
    {
        unwrap_or_log!(self.assert_open());
        let cmd = cmd.finalize(self.next_req_id);
        let cmd_len = cmd.length() as usize;
        cmd.serialize(self.buffer.as_mut_slice())?;

        self.send(cmd_len).await?;

        self.recv().await?;
        let mut retry_count = self.config.retry_count;

        loop {
            self.recv().await?;
            let ack = ack::AckPacket::parse(&self.buffer)?;
            self.verify_ack(&ack)?;

            if ack.ack_kind() == ack::AckKind::Pending {
                let pending_ack: ack::Pending = ack.ack_data_as()?;
                let waiting_time = pending_ack.waiting_time();
                task::sleep(waiting_time).await;
                retry_count -= 1;
                if retry_count == 0 {
                    return Err(ControlError::Io(anyhow::Error::msg(
                        "the number of times pending was returned exceeds the retry_count",
                    )));
                }
                continue;
            }

            self.next_req_id += 1;
            break;
        }

        ack::AckPacket::parse(&self.buffer)?
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
        timeout(self.timeout(), self.sock.send(&self.buffer[..len]))
            .await?
            .map_err(Into::into)
    }

    pub async fn recv(&mut self) -> ControlResult<usize> {
        timeout(self.timeout(), self.sock.recv(&mut self.buffer))
            .await?
            .map_err(Into::into)
    }
}

impl DeviceControl for ControlHandleInner {
    fn open(&mut self) -> ControlResult<()> {
        if self.is_opened() {
            return Ok(());
        }

        let bs = Bootstrap::new();
        match self.config.open_mode {
            OpenMode::Exclusive => {
                let ccp = ControlChannelPriviledge::new().enable_exclusive_access();
                bs.set_control_channel_priviledge(self, ccp)?;
            }

            OpenMode::Control => {
                let ccp = ControlChannelPriviledge::new().enable_control_access();
                bs.set_control_channel_priviledge(self, ccp)?;
            }

            OpenMode::MonitorAccess => {
                let ccp = bs.control_channel_priviledge(self)?;
                if ccp.is_exclusive_access_enabled() {
                    return Err(ControlError::Busy);
                }
            }
        }

        self.is_opened = true;
        Ok(())
    }

    fn close(&mut self) -> ControlResult<()> {
        if !self.is_opened() {
            return Ok(());
        }

        match self.config.open_mode {
            OpenMode::Exclusive | OpenMode::Control => {
                let bs = Bootstrap::new();
                let ccp = ControlChannelPriviledge::new();
                bs.set_control_channel_priviledge(self, ccp)?;
            }
            OpenMode::MonitorAccess => {}
        }

        self.is_opened = false;
        Ok(())
    }

    fn is_opened(&self) -> bool {
        self.is_opened
    }

    fn read_mem(&mut self, mut address: u64, buf: &mut [u8]) -> ControlResult<()> {
        let capability = self.capability.ok_or(ControlError::NotOpened)?;
        if buf.len() == 4 || !capability.is_write_mem_supported() {
            return self.read_reg_fallback(address, buf);
        }

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

    fn write_mem(&mut self, mut address: u64, data: &[u8]) -> ControlResult<()> {
        let capability = self.capability.ok_or(ControlError::NotOpened)?;
        if data.len() == 4 || !capability.is_write_mem_supported() {
            return self.write_reg_fallback(address, data);
        }

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
                let cmd = unwrap_or_log!(cmd::WriteMem::new(target_addr, &aligned_data));
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
        let capability = self.capability.ok_or(ControlError::NotOpened)?;
        let bs = Bootstrap::new();

        let url_string = if !capability.is_manifest_table_supported() {
            bs.first_url(self)?
        } else {
            let header = bs.manifest_header(self)?;
            let ent = header
                .entries(self)
                .filter_map(Result::ok)
                .max_by_key(|ent| ent.xml_file_version())
                .ok_or_else(|| {
                    ControlError::Io(anyhow::Error::msg(
                        "failed to retrieve `GigE` manifest entry",
                    ))
                })?;
            ent.url_string(self)?
        };

        let (xml, compression_type) = match XmlFileLocation::parse(&url_string)? {
            XmlFileLocation::Device {
                address,
                size,
                compression_type,
                ..
            } => {
                // Store current capacity so that we can set back it after XML retrieval because this needs exceptional large size of internal buffer.
                let current_capacity = self.buffer_capacity();
                let mut buf = vec![0; size as usize];
                unwrap_or_log!(self.read_mem(address, &mut buf));
                self.resize_buffer(current_capacity);
                (buf, compression_type)
            }

            XmlFileLocation::Net {
                url,
                compression_type,
            } => {
                let request = ureq::get(&url);
                let response =
                    unwrap_or_log!(request.call().map_err(|err| ControlError::Io(err.into())));
                if response.status() == 200 {
                    let mut buf = vec![];
                    unwrap_or_log!(response.into_reader().read_to_end(&mut buf));
                    (buf, compression_type)
                } else {
                    return Err(ControlError::Io(anyhow::Error::msg(format!(
                        "can't retrieve `GenApi` XML from vendor website: {:?}",
                        response
                    ))));
                }
            }

            XmlFileLocation::Host { .. } => {
                return Err(ControlError::NotSupported(
                    "can't retrieve `GenApi` XML from host storage".into(),
                ))
            }
        };

        match compression_type {
            CompressionType::Zip => {
                let xml = unwrap_or_log!(unzip_genxml(xml));
                Ok(String::from_utf8_lossy(&xml).into())
            }
            CompressionType::Uncompressed => Ok(String::from_utf8_lossy(&xml).into()),
        }
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
