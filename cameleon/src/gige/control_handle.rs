/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::{
    convert::TryInto,
    io::Read,
    sync::{Arc, Mutex},
    thread, time,
};

use async_std::{channel, future, net::UdpSocket, task};
use futures_channel::oneshot;
use futures_util::{select, FutureExt};

use cameleon_device::gige::protocol::{ack, cmd};

use crate::{
    genapi::CompressionType, utils::unzip_genxml, ControlError, ControlResult, DeviceControl,
};

use tracing::{debug, error};

use super::{
    register_map::{
        Bootstrap, ControlChannelPriviledge, GvcpCapability, StreamRegister, XmlFileLocation,
    },
    stream_handle::StreamParams,
};

const GVCP_DEFAULT_PORT: u16 = 3956;

/// Initial timeout duration for transaction between device and host.
/// This value is temporarily used until the device's bootstrap register value is read in
/// [`Device::Open`]
const INITIAL_TIMEOUT_DURATION: time::Duration = time::Duration::from_millis(500);

/// Default maximum retry count which determines how many times to retry when
/// pending acknowledge is returned from the device.
const DEFAULT_RETRY_COUNT: u16 = 3;

const GVCP_BUFFER_SIZE: usize = 1024;

pub type DeviceInfo = ack::Discovery;

pub struct ControlHandle {
    inner: Arc<Mutex<ControlHandleInner>>,
    event_tx: Option<channel::Sender<HeartbeatEvent>>,
    completion_rx: Option<oneshot::Receiver<()>>,
    info: DeviceInfo,
}

impl ControlHandle {
    pub fn new(info: DeviceInfo, stream_params: StreamParams) -> ControlResult<Self> {
        let inner = Arc::new(Mutex::new(task::block_on(ControlHandleInner::new(
            &info,
            stream_params,
        ))?));

        Ok(Self {
            inner,
            event_tx: None,
            completion_rx: None,
            info,
        })
    }

    pub fn set_heartbeat_timeout(&mut self, timeout: time::Duration) -> ControlResult<()> {
        unwrap_or_log!(Bootstrap::new().set_heartbeat_timeout(self, timeout));
        self.event_tx
            .as_ref()
            .map(|tx| tx.try_send(HeartbeatEvent::TimeoutChanged(timeout)));

        Ok(())
    }

    pub fn device_info(&self) -> &DeviceInfo {
        &self.info
    }

    pub fn bootstrap_register(&self) -> Bootstrap {
        Bootstrap::new()
    }
}

impl DeviceControl for ControlHandle {
    fn open(&mut self) -> ControlResult<()> {
        debug!("opening camera");
        let (heartbeat_timeout, need_heartbeat) = {
            let mut inner = self.inner.lock().unwrap();
            unwrap_or_log!(inner.open());

            let heartbeat_timeout = unwrap_or_log!(Bootstrap::new().heartbeat_timeout(&mut *inner));
            let need_heartbeat = matches!(
                inner.config.open_mode,
                OpenMode::Exclusive | OpenMode::Control
            );
            (heartbeat_timeout, need_heartbeat)
        };
        debug!("heartbeat timeout: {:#?}", heartbeat_timeout);
        let (event_tx, event_rx) = channel::unbounded();
        let (completion_tx, completion_rx) = oneshot::channel();
        let heartbeat_loop = HeartbeatLoop {
            inner: self.inner.clone(),
            timeout: heartbeat_timeout,
            event_rx,
            need_heartbeat,
        };

        self.event_tx = Some(event_tx);
        self.completion_rx = Some(completion_rx);

        thread::spawn(|| task::block_on(heartbeat_loop.run(completion_tx)));
        Ok(())
    }

    fn close(&mut self) -> ControlResult<()> {
        match (self.event_tx.take(), self.completion_rx.take()) {
            (Some(event_tx), Some(completion_rx)) => {
                event_tx.try_send(HeartbeatEvent::ChannelClosed).unwrap();
                task::block_on(completion_rx).ok();
            }
            (None, None) => {}
            _ => unreachable!(),
        }

        unwrap_or_log!(self.inner.lock().unwrap().close());
        Ok(())
    }

    fn is_opened(&self) -> bool {
        self.inner.lock().unwrap().is_opened()
    }

    fn read_mem(&mut self, address: u64, buf: &mut [u8]) -> ControlResult<()> {
        let mut inner = self.inner.lock().unwrap();
        unwrap_or_log!(assert_open(&mut *inner));
        unwrap_or_log!(inner.read_mem(address, buf));
        Ok(())
    }

    fn read_reg(&mut self, address: u64) -> ControlResult<[u8; 4]> {
        let mut inner = self.inner.lock().unwrap();
        unwrap_or_log!(assert_open(&mut *inner));
        Ok(unwrap_or_log!(inner.read_reg(address)))
    }

    fn write_mem(&mut self, address: u64, data: &[u8]) -> ControlResult<()> {
        let mut inner = self.inner.lock().unwrap();
        unwrap_or_log!(assert_open(&mut *inner));
        unwrap_or_log!(inner.write_mem(address, data));
        Ok(())
    }

    fn write_reg(&mut self, address: u64, data: [u8; 4]) -> ControlResult<()> {
        let mut inner = self.inner.lock().unwrap();
        unwrap_or_log!(assert_open(&mut *inner));
        unwrap_or_log!(inner.write_reg(address, data));
        Ok(())
    }

    fn genapi(&mut self) -> ControlResult<String> {
        let mut inner = self.inner.lock().unwrap();
        unwrap_or_log!(assert_open(&mut *inner));
        Ok(unwrap_or_log!(inner.genapi()))
    }

    fn enable_streaming(&mut self) -> ControlResult<()> {
        let mut inner = self.inner.lock().unwrap();
        unwrap_or_log!(assert_open(&mut *inner));
        unwrap_or_log!(inner.enable_streaming());
        Ok(())
    }

    fn disable_streaming(&mut self) -> ControlResult<()> {
        let mut inner = self.inner.lock().unwrap();
        unwrap_or_log!(assert_open(&mut *inner));
        unwrap_or_log!(inner.disable_streaming());
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenMode {
    Exclusive,
    Control,
    MonitorAccess,
}

impl Default for OpenMode {
    fn default() -> Self {
        OpenMode::Exclusive
    }
}

macro_rules! align {
    ($data:expr) => {
        ($data + 3) & !0b11
    };
}

#[derive(Debug)]
struct ControlHandleInner {
    sock: UdpSocket,
    config: ConnectionConfig,
    next_req_id: u16,
    buffer: Vec<u8>,
    capability: Option<GvcpCapability>,
    is_opened: bool,
    stream_params: StreamParams,
}

impl ControlHandleInner {
    async fn new(info: &DeviceInfo, stream_params: StreamParams) -> ControlResult<Self> {
        let sock = UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(|err| ControlError::Io(err.into()))?;
        let peer_ip = info.ip;
        sock.connect((peer_ip, GVCP_DEFAULT_PORT))
            .await
            .map_err(|err| ControlError::Io(err.into()))?;

        Ok(Self {
            sock,
            config: ConnectionConfig::default(),
            next_req_id: 1,
            buffer: vec![0; GVCP_BUFFER_SIZE],
            capability: None,
            is_opened: false,
            stream_params,
        })
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
        let cmd = cmd.finalize(self.next_req_id);
        let cmd_len = cmd.length() as usize;
        cmd.serialize(self.buffer.as_mut_slice())?;

        self.send(cmd_len).await?;

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

            self.next_req_id = self.next_req_id.checked_add(1).unwrap_or(1);
            break;
        }

        ack::AckPacket::parse(&self.buffer)?
            .ack_data_as()
            .map_err(Into::into)
    }

    async fn send(&self, len: usize) -> ControlResult<usize> {
        timeout(self.config.timeout, self.sock.send(&self.buffer[..len]))
            .await?
            .map_err(Into::into)
    }

    async fn recv(&mut self) -> ControlResult<usize> {
        timeout(self.config.timeout, self.sock.recv(&mut self.buffer))
            .await?
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

    fn capability(&mut self) -> ControlResult<GvcpCapability> {
        if let Some(capability) = self.capability {
            Ok(capability)
        } else {
            let capability = Bootstrap::new().gvcp_capability(self)?;
            self.capability = Some(capability);
            Ok(capability)
        }
    }
}

impl DeviceControl for ControlHandleInner {
    fn open(&mut self) -> ControlResult<()> {
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
        let capability = bs.gvcp_capability(self)?;
        if capability.is_pending_ack_supported() {
            let timeout = bs.pending_timeout(self)?;
            self.config.timeout = timeout;
        }

        self.is_opened = true;
        Ok(())
    }

    fn close(&mut self) -> ControlResult<()> {
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
        let capability = self.capability()?;
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

            let cmd = cmd::ReadMem::new(target_addr, aligned_read_len)?;
            let ack: ack::ReadMem = task::block_on(self.send_cmd(cmd))?;
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
        cmd.add_entry(address)?;
        let ack: ack::ReadReg = task::block_on(self.send_cmd(cmd))?;
        ack.iter()
            .next()
            .ok_or_else(|| ControlError::Io(anyhow::Error::msg("no entry in `ReadReg` ack packet")))
            .map(|v| *v)
    }

    fn write_mem(&mut self, mut address: u64, data: &[u8]) -> ControlResult<()> {
        let capability = self.capability()?;
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
                let cmd = cmd::WriteMem::new(target_addr, data_chunk)?;
                task::block_on(self.send_cmd(cmd))?
            } else {
                let mut aligned_data = vec![0; aligned_data_len];
                aligned_data[..data_chunk.len()].copy_from_slice(data_chunk);
                let cmd = cmd::WriteMem::new(target_addr, &aligned_data)?;
                task::block_on(self.send_cmd(cmd))?
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
        cmd.add_entry(cmd::WriteRegEntry::new(address, data)?)?;
        let ack: ack::WriteReg = task::block_on(self.send_cmd(cmd))?;

        if ack.entry_num() == 1 {
            Ok(())
        } else {
            Err(ControlError::Io(anyhow::Error::msg(
                "`WriteReg` failed: written entry num mismatch",
            )))
        }
    }

    fn genapi(&mut self) -> ControlResult<String> {
        let capability = self.capability()?;
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
        tracing::info!("retrieving GenICam file from: {}", url_string);

        let (xml, compression_type) = match XmlFileLocation::parse(&url_string)? {
            XmlFileLocation::Device {
                address,
                size,
                compression_type,
                ..
            } => {
                let mut buf = vec![0; size as usize];
                self.read_mem(address, &mut buf)?;
                (buf, compression_type)
            }

            XmlFileLocation::Net {
                url,
                compression_type,
            } => {
                let request = ureq::get(&url);
                let response = request.call().map_err(|err| ControlError::Io(err.into()))?;
                if response.status() == 200 {
                    let mut buf = vec![];
                    response.into_reader().read_to_end(&mut buf)?;
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
                let xml = unzip_genxml(xml)?;
                Ok(String::from_utf8_lossy(&xml).into())
            }
            CompressionType::Uncompressed => Ok(String::from_utf8_lossy(&xml).into()),
        }
    }

    fn enable_streaming(&mut self) -> ControlResult<()> {
        ensure!(
            Bootstrap::new().number_of_stream_channel(self)? == 1,
            ControlError::NotSupported("Number of stream channels other than 1".into())
        );

        let sr = StreamRegister::new(0);

        let packet_size = sr.packet_size(self)?;
        sr.set_packet_size(self, packet_size)?;

        let port = StreamRegister::new(0).channel_port(self)?;
        sr.set_channel_port(self, port.set_host_port(self.stream_params.host_port))?;

        sr.set_destination_address(self, self.stream_params.host_addr)?;

        Ok(())
    }

    fn disable_streaming(&mut self) -> ControlResult<()> {
        let port = StreamRegister::new(0).channel_port(self)?;
        StreamRegister::new(0).set_channel_port(self, port.set_host_port(0))?;
        Ok(())
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
    retry_count: u16,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            open_mode: OpenMode::default(),
            timeout: INITIAL_TIMEOUT_DURATION,
            retry_count: DEFAULT_RETRY_COUNT,
        }
    }
}

struct HeartbeatLoop {
    inner: Arc<Mutex<ControlHandleInner>>,
    timeout: time::Duration,
    event_rx: channel::Receiver<HeartbeatEvent>,
    need_heartbeat: bool,
}

impl HeartbeatLoop {
    async fn run(mut self, _completion_tx: oneshot::Sender<()>) {
        if self.need_heartbeat {
            loop {
                select! {
                    _ = task::sleep(self.timeout / 3).fuse() => {
                        let bs = Bootstrap::new();
                        debug!("reset heartbeat counter");
                        if let Err(err) = bs.control_channel_priviledge(&mut *self.inner.lock().unwrap()) {
                            error!("failed to reset heartbeat counter: {}", err);
                        }
                    }
                    event = self.event_rx.recv().fuse() => {
                        match event {
                            Ok(HeartbeatEvent::TimeoutChanged(timeout)) => self.timeout = timeout,
                            Ok(HeartbeatEvent::ChannelClosed) => break,
                            Err(err) => {
                                error!("failed to receive heartbeat event: {}", err);
                            }
                        }
                    }
                }
            }
        } else {
            loop {
                let event = self.event_rx.recv().await;
                match event {
                    Ok(HeartbeatEvent::ChannelClosed) => break,
                    Ok(_) => {}
                    Err(err) => {
                        error!("failed to receive heartbeat event: {}", err);
                    }
                }
            }
        }
    }
}

enum HeartbeatEvent {
    TimeoutChanged(time::Duration),
    ChannelClosed,
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

fn assert_open<Ctrl: DeviceControl>(device: Ctrl) -> ControlResult<()> {
    device
        .is_opened()
        .then(|| ())
        .ok_or(ControlError::NotOpened)
}
