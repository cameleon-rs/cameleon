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

use super::register_map::{self, Abrm, ManifestTable, Sbrm, Sirm};

use crate::{camera::DeviceControl, DeviceError, DeviceResult};

/// Initial timeout duration for transaction between device and host.
/// This value is temporarily used until the device's bootstrap register value is read.
const INITIAL_TIMEOUT_DURATION: Duration = Duration::from_millis(500);

/// Initial maximum command  packet length for transaction between device and host.
/// This value is temporarily used until the device's bootstrap register value is read.
const INITIAL_MAXIMUM_CMD_LENGTH: u32 = 128;

/// Initial maximum acknowledge packet length for transaction between device and host.
/// This value is temporarily used until the device's bootstrap register value is read.
const INITIAL_MAXIMUM_ACK_LENGTH: u32 = 128;

/// This handle provides low level API to read and write data from the device.  
/// See [`ControlHandle::abrm`] and [`register_map`](super::register_map) which provide more
/// convenient API to communicate with the device.
///
///
/// # Examples
///
/// ```no_run
/// use cameleon::u3v::enumerate_devices;
///
/// // Enumerate devices connected to the host.
/// let mut devices = enumerate_devices().unwrap();
///
/// // If no device is found, return.
/// if devices.is_empty() {
///     return;
/// }
///
/// let mut device = devices.pop().unwrap();
///
/// // Obtain and open control handle.
/// let handle = device.control_handle();
/// handle.open().unwrap();
///
/// // Read 64bytes from address 0x0184.
/// let address = 0x0184;
/// let mut buffer = vec![0; 64];
/// handle.read_mem(address, &mut buffer).unwrap();
/// ```
pub struct ControlHandle {
    inner: u3v::ControlChannel,
    config: ConnectionConfig,
    /// Request id of the next packet.
    next_req_id: u16,
    /// Buffer for serializing/deserializing a packet.
    buffer: Vec<u8>,

    /// Cache for `Sbrm` address.
    sbrm_addr: Option<u64>,

    /// Cache for `Sirm` address.
    sirm_addr: Option<u64>,

    /// Cache for `ManifestTable` address.
    manifest_addr: Option<u64>,
}

impl ControlHandle {
    /// Capacity of the buffer inside [`ControlHandleImpl`], the buffer is used for
    /// serializing/deserializing packet. This buffer automatically extend according to packet
    /// length.
    pub fn buffer_capacity(&self) -> usize {
        self.buffer.capacity()
    }

    /// Resize the capacity of the buffer inside [`ControlHandleImpl`], the buffer is used for
    /// serializing/deserializing packet. This buffer automatically extend according to packet
    /// length.
    pub fn resize_buffer(&mut self, size: usize) {
        self.buffer.resize(size, 0);
        self.buffer.shrink_to_fit();
    }

    /// Timeout duration of each transaction between device.
    ///
    /// NOTE: [`ControlHandle::read_mem`] and [`ControlHandle::write_mem`] may send multiple
    /// requests in a single call. In that case, Timeout is reflected to each request.
    #[must_use]
    pub fn timeout_duration(&self) -> Duration {
        self.config.timeout_duration
    }

    /// Set timeout duration of each transaction between device.
    ///
    /// NOTE: [`ControlHandle::read_mem`] and [`ControlHandle::write_mem`] may send multiple
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

    /// Return [`Abrm`].
    pub fn abrm(&mut self) -> DeviceResult<Abrm> {
        Abrm::new(self)
    }

    /// Return [`Sbrm`].
    pub fn sbrm(&mut self) -> DeviceResult<Sbrm> {
        let addr = if let Some(addr) = self.sbrm_addr {
            addr
        } else {
            let addr = self.abrm()?.sbrm_address(self)?;
            self.sbrm_addr = Some(addr);
            addr
        };

        Sbrm::new(self, addr)
    }

    /// Return [`Sirm`].
    pub fn sirm(&mut self) -> DeviceResult<Sirm> {
        let addr = if let Some(addr) = self.sirm_addr {
            addr
        } else {
            let addr = self.sbrm()?.sirm_address(self)?.ok_or_else(|| {
                DeviceError::InternalError("the u3v device doesn't have `SIRM ADDRESS`".into())
            })?;
            self.sirm_addr = Some(addr);
            addr
        };

        Ok(Sirm::new(addr))
    }

    /// Return [`ManifestTable`].
    pub fn manifest_table(&mut self) -> DeviceResult<ManifestTable> {
        let addr = if let Some(addr) = self.manifest_addr {
            addr
        } else {
            let addr = self.abrm()?.manifest_table_address(self)?;
            self.manifest_addr = Some(addr);
            addr
        };

        Ok(ManifestTable::new(addr))
    }

    pub(super) fn new(device: &u3v::Device) -> DeviceResult<Self> {
        let inner = device.control_channel()?;

        Ok(Self {
            inner,
            config: ConnectionConfig::default(),
            next_req_id: 0,
            buffer: Vec::new(),
            sbrm_addr: None,
            sirm_addr: None,
            manifest_addr: None,
        })
    }

    fn assert_open(&self) -> DeviceResult<()> {
        if self.is_opened() {
            Ok(())
        } else {
            Err(DeviceError::NotOpened)
        }
    }

    fn initialize_config(&mut self) -> DeviceResult<()> {
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

    fn send_cmd<'a, T, U>(&'a mut self, cmd: T) -> DeviceResult<U>
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

            self.next_req_id += 1;
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
            Err(DeviceError::Io(
                "the number of times pending was returned exceeds the retry_count.".into(),
            ))
        }
    }

    fn verify_ack(&self, ack: &ack::AckPacket) -> DeviceResult<()> {
        let status = ack.status().kind();
        if status != ack::StatusKind::GenCp(ack::GenCpStatus::Success) {
            return Err(DeviceError::Io(
                format!("invalid status: {:?}", ack.status().kind()).into(),
            ));
        }

        if ack.request_id() != self.next_req_id {
            return Err(DeviceError::Io("request id mismatch".into()));
        }

        Ok(())
    }

    fn verify_xml(&mut self, xml: &[u8], ent: register_map::ManifestEntry) -> DeviceResult<()> {
        use sha1::Digest;

        if let Some(hash) = ent.sha1_hash(self)? {
            let xml_hash = sha1::Sha1::digest(xml);
            if xml_hash.as_slice() != hash {
                Err(DeviceError::InternalError(
                    "sha1 of retrieved xml file isn't same as entry's hash".into(),
                ))
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }
}

impl DeviceControl for ControlHandle {
    fn open(&mut self) -> DeviceResult<()> {
        if self.is_opened() {
            return Ok(());
        }

        self.inner.open()?;
        // Clean up control channel state.
        self.inner.set_halt(self.config.timeout_duration)?;
        self.inner.clear_halt()?;
        self.initialize_config()?;

        Ok(())
    }

    fn is_opened(&self) -> bool {
        self.inner.is_opened()
    }

    fn close(&mut self) -> DeviceResult<()> {
        if self.is_opened() {
            Ok(self.inner.close()?)
        } else {
            Ok(())
        }
    }

    fn write_mem(&mut self, address: u64, data: &[u8]) -> DeviceResult<()> {
        self.assert_open()?;

        let cmd = cmd::WriteMem::new(address, data)?;
        let maximum_cmd_length = self.config.maximum_cmd_length;

        for chunk in cmd.chunks(maximum_cmd_length as usize).unwrap() {
            let chunk_data_len = chunk.data_len();
            let ack: ack::WriteMem = self.send_cmd(chunk)?;

            if ack.length as usize != chunk_data_len {
                return Err(DeviceError::Io(
                    "write mem failed: written length mismatch".into(),
                ));
            }
        }

        Ok(())
    }

    fn read_mem(&mut self, mut address: u64, buf: &mut [u8]) -> DeviceResult<()> {
        self.assert_open()?;

        // Chunks buffer if buffer length is larger than u16::MAX.
        for buf_chunk in buf.chunks_mut(std::u16::MAX as usize) {
            // Create command for buffer chunk.
            let cmd = cmd::ReadMem::new(address, buf_chunk.len().try_into().unwrap());
            let maximum_ack_length = self.config.maximum_ack_length;

            // Chunks command so that each acknowledge packet length fits to maximum_ack_length.
            let mut total_read_len = 0;
            for cmd_chunk in cmd.chunks(maximum_ack_length as usize).unwrap() {
                let read_len = cmd_chunk.read_length();
                let ack: ack::ReadMem = self.send_cmd(cmd_chunk)?;
                (&mut buf_chunk[total_read_len..total_read_len + read_len as usize])
                    .copy_from_slice(ack.data);
                total_read_len += read_len as usize;
            }

            address += buf_chunk.len() as u64;
        }

        Ok(())
    }

    fn gen_api(&mut self) -> DeviceResult<String> {
        let table = self.manifest_table()?;
        // Use newest version if there are more than one entries.
        let mut newest_ent = None;
        for ent in table.entries(self)? {
            let file_info = ent.file_info(self)?;
            if file_info.file_type()? == register_map::GenICamFileType::DeviceXml {
                let version = ent.genicam_file_version(self)?;
                match &newest_ent {
                    Some((_, cur_version, _)) if &version <= cur_version => {
                        // Current entry is newest.
                    }
                    _ => newest_ent = Some((ent, version, file_info)),
                }
            }
        }

        let (ent, _, file_info) = newest_ent.ok_or_else(|| {
            DeviceError::InternalError("device doesn't have valid `ManifestEntry`".into())
        })?;

        let file_address: u64 = ent.file_address(self)?;
        let file_size: usize = ent.file_size(self)?.try_into()?;
        let comp_type = file_info.compression_type()?;

        // Store current capacity so that we can set back it after XML retrieval because this needs exceptional large size of internal buffer.
        let current_capacity = self.buffer_capacity();
        let mut buf = vec![0; file_size];
        self.read_mem(file_address, &mut buf)?;
        self.resize_buffer(current_capacity);

        // Verify retrieved xml has correct hash.
        self.verify_xml(&buf, ent)?;

        fn zip_err(err: impl std::fmt::Debug) -> DeviceError {
            DeviceError::InternalError(format!("zipped xml file is broken: {:?}", err).into())
        }
        match comp_type {
            register_map::CompressionType::Zip => {
                let mut zip = zip::ZipArchive::new(std::io::Cursor::new(buf)).unwrap();
                if zip.len() != 1 {
                    return Err(zip_err("more than one files in zipped GenApi XML"));
                }
                let mut file = zip.by_index(0).map_err(zip_err)?;
                let mut xml = vec![0; file.size().try_into()?];
                file.read_to_end(&mut xml).map_err(zip_err)?;
                Ok(String::from_utf8_lossy(&xml).into())
            }

            register_map::CompressionType::Uncompressed => Ok(String::from_utf8_lossy(&buf).into()),
        }
    }

    fn enable_streaming(&mut self) -> DeviceResult<()> {
        let sirm = self.sirm()?;
        sirm.enable_stream(self)
    }

    fn disable_streaming(&mut self) -> DeviceResult<()> {
        let sirm = self.sirm()?;
        sirm.disable_stream(self)
    }
}

/// Thread safe version of [`ContolHandle`].
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
            fn $method(&mut $self, $($arg: $arg_ty),*) -> $ret_ty {
                $self.0.lock().unwrap().$method($($arg),*)
            }
        )*
    }
}

impl SharedControlHandle {
    impl_shared_control_handle!(
        /// Thread safe version of [`ContolHandle::buffer_capacity`].
        #[must_use]
        pub fn buffer_capacity(&self) -> usize,
        /// Thread safe version of [`ContolHandle::resize_buffer`].
        pub fn resize_buffer(&self, size: usize) -> (),
        #[must_use]
        /// Thread safe version of [`ContolHandle::timeout_duration`].
        #[must_use]
        pub fn timeout_duration(&self) -> Duration,
        /// Thread safe version of [`ContolHandle::set_timeout_duration`].
        pub fn set_timeout_duration(&self, duration: Duration) -> (),
        /// Thread safe version of [`ContolHandle::retry_count`].
        #[must_use]
        pub fn retry_count(&self) -> u16,
        /// Thread safe version of [`ContolHandle::set_retry_count`].
        pub fn set_retry_count(&self, count: u16) -> (),
        /// Thread safe version of [`ContolHandle::abrm`].
        pub fn abrm(&self) -> DeviceResult<Abrm>,
        /// Thread safe version of [`ContolHandle::sbrm`].
        pub fn sbrm(&self) -> DeviceResult<Sbrm>,
        /// Thread safe version of [`ContolHandle::sirm`].
        pub fn sirm(&self) -> DeviceResult<Sirm>,
        /// Thread safe version of [`ContolHandle::manifest_table`].
        pub fn manifest_table(&self) -> DeviceResult<ManifestTable>

    );
}

impl DeviceControl for SharedControlHandle {
    impl_shared_control_handle! {
        fn is_opened(&self) -> bool
    }

    impl_shared_control_handle! {
        fn open(&mut self) -> DeviceResult<()>,
        fn close(&mut self) -> DeviceResult<()>,
        fn read_mem(&mut self, address: u64, buf: &mut [u8]) -> DeviceResult<()>,
        fn write_mem(&mut self, address: u64, data: &[u8]) -> DeviceResult<()>,
        fn gen_api(&mut self) -> DeviceResult<String>,
        fn enable_streaming(&mut self) -> DeviceResult<()>,
        fn disable_streaming(&mut self) -> DeviceResult<()>
    }
}

/// A marker trait represents the handle is for `U3V`.
pub trait U3VDeviceControl: DeviceControl {}
impl U3VDeviceControl for ControlHandle {}
impl U3VDeviceControl for SharedControlHandle {}

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
