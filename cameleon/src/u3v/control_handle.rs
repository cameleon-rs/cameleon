/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module contains low level device control implementation for `U3V` device.

use std::{
    convert::TryInto,
    io::Read,
    sync::{Arc, Mutex},
    time::Duration,
};

use cameleon_device::{
    u3v,
    u3v::protocol::{ack, cmd},
};
use tracing::error;

use super::register_map::{self, Abrm, ManifestTable, Sbrm, Sirm};

use crate::{camera::DeviceControl, genapi::CompressionType, ControlError, ControlResult};

/// Initial timeout duration for transaction between device and host.
/// This value is temporarily used until the device's bootstrap register value is read.
const INITIAL_TIMEOUT_DURATION: Duration = Duration::from_millis(500);

/// Initial maximum command  packet length for transaction between device and host.
/// This value is temporarily used until the device's bootstrap register value is read.
const INITIAL_MAXIMUM_CMD_LENGTH: u32 = 128;

/// Initial maximum acknowledge packet length for transaction between device and host.
/// This value is temporarily used until the device's bootstrap register value is read.
const INITIAL_MAXIMUM_ACK_LENGTH: u32 = 128;

const PAYLOAD_TRANSFER_SIZE: u32 = 1024 * 64;

/// This handle provides low level API to read and write data from the device.  
/// See [`ControlHandle::abrm`] and [`register_map`](super::register_map) which provide more
/// convenient way to communicate with `u3v` specific registers.
///
/// # Examples
///
/// ```no_run
/// use cameleon::{Camera, DeviceControl};
/// use cameleon::u3v;
/// use cameleon::genapi;
///
/// // Enumerates cameras connected to the host.
/// let mut cameras = u3v::enumerate_cameras().unwrap();
///
/// // If no camera is found, return.
/// if cameras.is_empty() {
///     return;
/// }
/// let mut camera = cameras.pop().unwrap();
///
/// // Opens the camera.
/// camera.open().unwrap();
///
/// // Read 64bytes from address 0x0184.
/// let address = 0x0184;
/// let mut buffer = vec![0; 64];
/// camera.ctrl.read(address, &mut buffer).unwrap();
/// ```
pub struct ControlHandle {
    inner: u3v::ControlChannel,
    config: ConnectionConfig,
    /// Request id of the next packet.
    next_req_id: u16,
    /// Buffer for serializing/deserializing a packet.
    buffer: Vec<u8>,

    /// Device information.
    info: u3v::DeviceInfo,

    /// Cache for `Abrm`.
    abrm: Option<Abrm>,
    /// Cache for `Sbrm`.
    sbrm: Option<Sbrm>,
    /// Cache for `Sirm`.
    sirm: Option<Sirm>,
    /// Cache for `ManifestTable`.
    manifest_table: Option<ManifestTable>,
}

impl ControlHandle {
    /// Capacity of the buffer inside [`ControlHandle`], the buffer is used for
    /// serializing/deserializing packet. This buffer automatically extend according to packet
    /// length.
    pub fn buffer_capacity(&self) -> usize {
        self.buffer.capacity()
    }

    /// Resize the capacity of the buffer inside [`ControlHandle`], the buffer is used for
    /// serializing/deserializing packet. This buffer automatically extend according to packet
    /// length.
    pub fn resize_buffer(&mut self, size: usize) {
        self.buffer.resize(size, 0);
        self.buffer.shrink_to_fit();
    }

    /// Timeout duration of each transaction between device.
    ///
    /// NOTE: [`ControlHandle::read`] and [`ControlHandle::write`] may send multiple
    /// requests in a single call. In that case, Timeout is reflected to each request.
    #[must_use]
    pub fn timeout_duration(&self) -> Duration {
        self.config.timeout_duration
    }

    /// Set timeout duration of each transaction between device.
    ///
    /// NOTE: [`ControlHandle::read`] and [`ControlHandle::write`] may send multiple
    /// requests in a single call. In that case, Timeout is reflected to each request.
    ///
    /// In normal use case, no need to modify timeout duration.
    pub fn set_timeout_duration(&mut self, duration: Duration) {
        self.config.timeout_duration = duration;
    }

    /// The value determines how many times to retry when pending acknowledge is returned from the
    /// device.
    #[must_use]
    pub fn retry_count(&self) -> u16 {
        self.config.retry_count
    }

    /// Set the value determines how many times to retry when pending acknowledge is returned from the
    /// device.
    pub fn set_retry_count(&mut self, count: u16) {
        self.config.retry_count = count;
    }

    /// Returns the device info of the handle.
    pub fn device_info(&self) -> &u3v::DeviceInfo {
        &self.info
    }

    /// Returns [`Abrm`].
    pub fn abrm(&mut self) -> ControlResult<Abrm> {
        if let Some(abrm) = self.abrm {
            return Ok(abrm);
        }
        let abrm = Abrm::new(self)?;
        self.abrm = Some(abrm);

        Ok(abrm)
    }

    /// Returns [`Sbrm`].
    pub fn sbrm(&mut self) -> ControlResult<Sbrm> {
        if let Some(sbrm) = self.sbrm {
            return Ok(sbrm);
        }
        let addr = self.abrm()?.sbrm_address(self)?;
        let sbrm = Sbrm::new(self, addr)?;
        self.sbrm = Some(sbrm);
        Ok(sbrm)
    }

    /// Returns [`Sirm`].
    pub fn sirm(&mut self) -> ControlResult<Sirm> {
        if let Some(sirm) = self.sirm {
            return Ok(sirm);
        }

        let addr = self.sbrm()?.sirm_address(self)?.ok_or_else(|| {
            ControlError::InvalidDevice("the u3v device doesn't have `SIRM ADDRESS`".into())
        })?;
        let sirm = Sirm::new(addr);
        self.sirm = Some(sirm);

        Ok(sirm)
    }

    /// Returns [`ManifestTable`].
    pub fn manifest_table(&mut self) -> ControlResult<ManifestTable> {
        if let Some(manifest_table) = self.manifest_table {
            return Ok(manifest_table);
        }

        let addr = self.abrm()?.manifest_table_address(self)?;
        let manifest_table = ManifestTable::new(addr);
        self.manifest_table = Some(manifest_table);
        Ok(manifest_table)
    }

    pub(super) fn new(device: &u3v::Device) -> ControlResult<Self> {
        let inner = device.control_channel()?;

        Ok(Self {
            inner,
            config: ConnectionConfig::default(),
            next_req_id: 0,
            buffer: Vec::new(),
            info: device.device_info.clone(),
            abrm: None,
            sbrm: None,
            sirm: None,
            manifest_table: None,
        })
    }

    fn assert_open(&self) -> ControlResult<()> {
        if self.is_opened() {
            Ok(())
        } else {
            Err(ControlError::NotOpened)
        }
    }

    fn initialize_config(&mut self) -> ControlResult<()> {
        let abrm = self.abrm()?;
        let sbrm = abrm.sbrm(self)?;

        let timeout_duration = abrm.maximum_device_response_time(self)?;
        let maximum_cmd_length = sbrm.maximum_command_transfer_length(self)?;
        let maximum_ack_length = sbrm.maximum_acknowledge_trasfer_length(self)?;

        self.config.timeout_duration = timeout_duration;
        self.config.maximum_cmd_length = maximum_cmd_length;
        self.config.maximum_ack_length = maximum_ack_length;

        Ok(())
    }

    fn send_cmd<'a, T, U>(&'a mut self, cmd: T) -> ControlResult<U>
    where
        T: cmd::CommandScd,
        U: ack::ParseScd<'a>,
    {
        let cmd = cmd.finalize(self.next_req_id);
        let cmd_len = cmd.cmd_len();
        let ack_len = cmd.maximum_ack_len();
        if self.buffer.len() < std::cmp::max(cmd_len, ack_len) {
            self.buffer.resize(std::cmp::max(cmd_len, ack_len), 0);
        }

        // Serialize and send command.
        cmd.serialize(self.buffer.as_mut_slice())?;
        self.inner
            .send(&self.buffer[..cmd_len], self.config.timeout_duration)?;

        // Receive ack and interpret the packet.
        let mut retry_count = self.config.retry_count;
        let mut ok = None;
        while retry_count > 0 {
            let recv_len = self
                .inner
                .recv(&mut self.buffer, self.config.timeout_duration)?;

            let ack = ack::AckPacket::parse(&self.buffer[0..recv_len])?;
            self.verify_ack(&ack)?;

            // Retry up to retry count.
            if ack.scd_kind() == ack::ScdKind::Pending {
                let pending_ack: ack::Pending = ack.scd_as()?;
                std::thread::sleep(pending_ack.timeout);
                retry_count -= 1;
                continue;
            }

            self.next_req_id = self.next_req_id.wrapping_add(1);
            ok = Some(recv_len);
            break;
        }

        // This codes seems weird due to a lifetime problem.
        // `ack::AckPacket::parse` is a fast operation, so it's ok to call it repeatedly.
        if let Some(recv_len) = ok {
            Ok(ack::AckPacket::parse(&self.buffer[0..recv_len])
                .unwrap()
                .scd_as()?)
        } else {
            Err(ControlError::Io(anyhow::Error::msg(
                "the number of times pending was returned exceeds the retry_count.",
            )))
        }
    }

    fn verify_ack(&self, ack: &ack::AckPacket) -> ControlResult<()> {
        let status = ack.status().kind();
        if status != ack::StatusKind::GenCp(ack::GenCpStatus::Success) {
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

    fn verify_xml(&mut self, xml: &[u8], ent: register_map::ManifestEntry) -> ControlResult<()> {
        use sha1::Digest;

        if let Some(hash) = ent.sha1_hash(self)? {
            let xml_hash = sha1::Sha1::digest(xml);
            if xml_hash.as_slice() == hash {
                Ok(())
            } else {
                Err(ControlError::InvalidDevice(
                    "sha1 of retrieved xml file isn't same as entry's hash".into(),
                ))
            }
        } else {
            Ok(())
        }
    }
}

macro_rules! unwrap_or_log {
    ($expr:expr) => {{
        match $expr {
            Ok(v) => v,
            Err(error) => {
                error!(?error);
                return Err(error.into());
            }
        }
    }};
}

impl DeviceControl for ControlHandle {
    fn open(&mut self) -> ControlResult<()> {
        if self.is_opened() {
            return Ok(());
        }

        unwrap_or_log!(self.inner.open());
        // Clean up control channel state.
        unwrap_or_log!(self.inner.set_halt(self.config.timeout_duration));
        unwrap_or_log!(self.inner.clear_halt());
        unwrap_or_log!(self.initialize_config());

        Ok(())
    }

    fn is_opened(&self) -> bool {
        self.inner.is_opened()
    }

    fn close(&mut self) -> ControlResult<()> {
        if self.is_opened() {
            unwrap_or_log!(self.inner.close());
        }
        Ok(())
    }

    fn write(&mut self, address: u64, data: &[u8]) -> ControlResult<()> {
        unwrap_or_log!(self.assert_open());

        let cmd = unwrap_or_log!(cmd::WriteMem::new(address, data));
        let maximum_cmd_length = self.config.maximum_cmd_length;

        for chunk in cmd.chunks(maximum_cmd_length as usize).unwrap() {
            let chunk_data_len = chunk.data_len();
            let ack: ack::WriteMem = unwrap_or_log!(self.send_cmd(chunk));

            if ack.length as usize != chunk_data_len {
                let err_msg = "write mem failed: written length mismatch";
                return Err(ControlError::Io(anyhow::Error::msg(err_msg)));
            }
        }

        Ok(())
    }

    fn read(&mut self, mut address: u64, buf: &mut [u8]) -> ControlResult<()> {
        unwrap_or_log!(self.assert_open());

        // Chunks buffer if buffer length is larger than maximum read length calculated from
        // maximum ack length.
        for buf_chunk in buf.chunks_mut(cmd::ReadMem::maximum_read_length(
            self.config.maximum_ack_length as usize,
        ) as usize)
        {
            let read_len: u16 = buf_chunk.len().try_into().unwrap();

            let cmd = cmd::ReadMem::new(address, read_len);
            let ack: ack::ReadMem = unwrap_or_log!(self.send_cmd(cmd));
            buf_chunk.copy_from_slice(ack.data);
            address += read_len as u64;
        }

        Ok(())
    }

    fn genapi(&mut self) -> ControlResult<String> {
        fn zip_err(err: impl std::fmt::Debug) -> ControlError {
            ControlError::InvalidDevice(format!("zipped xml file is broken: {:?}", err).into())
        }

        let table = unwrap_or_log!(self.manifest_table());
        // Use newest version if there are more than one entries.
        let mut newest_ent = None;
        for ent in unwrap_or_log!(table.entries(self)) {
            let file_info = unwrap_or_log!(ent.file_info(self));
            if unwrap_or_log!(file_info.file_type()) == register_map::GenICamFileType::DeviceXml {
                let version = unwrap_or_log!(ent.genicam_file_version(self));
                match &newest_ent {
                    Some((_, cur_version, _)) if &version <= cur_version => {
                        // Current entry is newest.
                    }
                    _ => newest_ent = Some((ent, version, file_info)),
                }
            }
        }

        let (ent, _, file_info) = unwrap_or_log!(newest_ent.ok_or_else(|| {
            ControlError::InvalidDevice("device doesn't have valid `ManifestEntry`".into())
        }));

        let file_address: u64 = unwrap_or_log!(ent.file_address(self));
        let file_size: usize = unwrap_or_log!(unwrap_or_log!(ent.file_size(self)).try_into());
        let comp_type = unwrap_or_log!(file_info.compression_type());

        // Store current capacity so that we can set back it after XML retrieval because this needs exceptional large size of internal buffer.
        let current_capacity = self.buffer_capacity();
        let mut buf = vec![0; file_size];
        unwrap_or_log!(self.read(file_address, &mut buf));
        self.resize_buffer(current_capacity);

        // Verify retrieved xml has correct hash.
        unwrap_or_log!(self.verify_xml(&buf, ent));

        match comp_type {
            CompressionType::Zip => {
                let mut zip = zip::ZipArchive::new(std::io::Cursor::new(buf)).unwrap();
                if zip.len() != 1 {
                    return Err(zip_err("more than one files in zipped GenApi XML"));
                }
                let mut file = unwrap_or_log!(zip.by_index(0).map_err(zip_err));
                let file_size: usize = unwrap_or_log!(file.size().try_into());
                let mut xml = Vec::with_capacity(file_size);
                unwrap_or_log!(file.read_to_end(&mut xml).map_err(zip_err));
                Ok(String::from_utf8_lossy(&xml).into())
            }

            CompressionType::Uncompressed => Ok(String::from_utf8_lossy(&buf).into()),
        }
    }

    fn enable_streaming(&mut self) -> ControlResult<()> {
        let sirm = unwrap_or_log!(self.sirm());

        let payload_alignment = unwrap_or_log!(sirm.payload_size_alignment(self));
        macro_rules! align {
            ($expr:expr, $ty: ty) => {
                // Payload alignment is always power of two.
                ($expr + (payload_alignment as $ty - 1)) & !(payload_alignment as $ty - 1)
            };
        }

        let required_leader_size = unwrap_or_log!(sirm.required_leader_size(self));
        let required_payload_size = unwrap_or_log!(sirm.required_payload_size(self));
        let required_trailer_size = unwrap_or_log!(sirm.required_leader_size(self));

        let payload_transfer_size = align!(PAYLOAD_TRANSFER_SIZE, u32);
        let payload_transfer_count = (required_payload_size / payload_transfer_size as u64) as u32;
        let payload_final_transfer1_size =
            align!(required_payload_size % payload_transfer_size as u64, u64) as u32;
        let payload_final_transfer2_size = 0;

        let maximum_leader_size = if required_leader_size == 0 {
            payload_transfer_size
        } else {
            align!(required_leader_size, u32)
        };
        let maximum_trailer_size = if required_trailer_size == 0 {
            payload_transfer_size
        } else {
            align!(required_trailer_size, u32)
        };

        unwrap_or_log!(sirm.set_payload_transfer_size(self, payload_transfer_size));
        unwrap_or_log!(sirm.set_payload_transfer_count(self, payload_transfer_count));
        unwrap_or_log!(sirm.set_payload_final_transfer1_size(self, payload_final_transfer1_size));
        unwrap_or_log!(sirm.set_payload_final_transfer2_size(self, payload_final_transfer2_size));
        unwrap_or_log!(sirm.set_maximum_leader_size(self, maximum_leader_size));
        unwrap_or_log!(sirm.set_maximum_trailer_size(self, maximum_trailer_size));
        unwrap_or_log!(sirm.enable_stream(self));

        Ok(())
    }

    fn disable_streaming(&mut self) -> ControlResult<()> {
        let sirm = unwrap_or_log!(self.sirm());
        sirm.disable_stream(self)
    }
}

impl Drop for ControlHandle {
    fn drop(&mut self) {
        if let Err(e) = self.close() {
            error!(?e)
        }
    }
}

/// Thread safe version of [`ControlHandle`].
#[derive(Clone)]
pub struct SharedControlHandle(Arc<Mutex<ControlHandle>>);

macro_rules! impl_shared_control_handle {
    ($(
            $(#[$meta:meta])*
            $vis:vis fn $method:ident(&$self:ident $(,$arg:ident: $arg_ty:ty)*) -> $ret_ty:ty),*) => {
        $(
            $(#[$meta])*
            $vis fn $method(&$self, $($arg: $arg_ty),*) -> $ret_ty {
                $self.0.lock().unwrap().$method($($arg),*)
            }
        )*
    };

    ($(
            $(#[$meta:meta])*
            $vis:vis fn $method:ident(&mut $self:ident $(,$arg:ident: $arg_ty:ty)*) -> $ret_ty:ty),*) => {
        $(
            $(#[$meta])*
            $vis fn $method(&mut $self, $($arg: $arg_ty),*) -> $ret_ty {
                $self.0.lock().unwrap().$method($($arg),*)
            }
        )*
    }
}

impl From<ControlHandle> for SharedControlHandle {
    fn from(handle: ControlHandle) -> Self {
        Self(Arc::new(Mutex::new(handle)))
    }
}

impl SharedControlHandle {
    impl_shared_control_handle!(
        /// Thread safe version of [`ControlHandle::buffer_capacity`].
        #[must_use]
        pub fn buffer_capacity(&self) -> usize,
        /// Thread safe version of [`ControlHandle::resize_buffer`].
        pub fn resize_buffer(&self, size: usize) -> (),
        #[must_use]
        /// Thread safe version of [`ControlHandle::timeout_duration`].
        #[must_use]
        pub fn timeout_duration(&self) -> Duration,
        /// Thread safe version of [`ControlHandle::set_timeout_duration`].
        pub fn set_timeout_duration(&self, duration: Duration) -> (),
        /// Thread safe version of [`ControlHandle::retry_count`].
        #[must_use]
        pub fn retry_count(&self) -> u16,
        /// Thread safe version of [`ControlHandle::set_retry_count`].
        pub fn set_retry_count(&self, count: u16) -> ()
    );

    /// Returns the device info of the handle.
    pub fn device_info(&self) -> u3v::DeviceInfo {
        self.0.lock().unwrap().device_info().clone()
    }
}

impl DeviceControl for SharedControlHandle {
    impl_shared_control_handle! {
        fn is_opened(&self) -> bool
    }

    impl_shared_control_handle! {
        fn open(&mut self) -> ControlResult<()>,
        fn close(&mut self) -> ControlResult<()>,
        fn read(&mut self, address: u64, buf: &mut [u8]) -> ControlResult<()>,
        fn write(&mut self, address: u64, data: &[u8]) -> ControlResult<()>,
        fn genapi(&mut self) -> ControlResult<String>,
        fn enable_streaming(&mut self) -> ControlResult<()>,
        fn disable_streaming(&mut self) -> ControlResult<()>
    }
}

struct ConnectionConfig {
    /// Timeout duration of each transaction between device.
    timeout_duration: Duration,

    /// The value determines how many times to retry when pending acknowledge is returned from the
    /// device.
    retry_count: u16,

    /// Maximum length of a command sent to device from host. Unit is byte.
    maximum_cmd_length: u32,

    /// Maximum length of a acknowledge sent to host from device. Unit is byte.
    maximum_ack_length: u32,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            timeout_duration: INITIAL_TIMEOUT_DURATION,
            retry_count: 3,
            maximum_cmd_length: INITIAL_MAXIMUM_CMD_LENGTH,
            maximum_ack_length: INITIAL_MAXIMUM_ACK_LENGTH,
        }
    }
}

impl From<SharedControlHandle> for Box<dyn DeviceControl> {
    fn from(ctrl: SharedControlHandle) -> Self {
        Box::new(ctrl)
    }
}

impl From<ControlHandle> for Box<dyn DeviceControl> {
    fn from(ctrl: ControlHandle) -> Self {
        Box::new(ctrl)
    }
}
