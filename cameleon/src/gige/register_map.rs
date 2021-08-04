/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! GigE device register structs.
//!
//! This module abstracts physical configuration of the device and provides an easy access to
//! its registers.
//!
pub use cameleon_device::gige::register_map::{
    ControlChannelPriviledge, DeviceMode, GvcpCapability, GvspCapability, MessageChannelCapability,
    NicCapability, NicConfiguration, StreamChannelPort, StreamPacketSize, XmlFileLocation,
};

use std::{convert::TryInto, net::Ipv4Addr, time};

use cameleon_device::gige::register_map::{self, bootstrap, stream};
use cameleon_impl::bytes_io::{BytesConvertible, ReadBytes, StaticString, WriteBytes};
use semver::Version;

use crate::{ControlError, ControlResult, DeviceControl};

/// Represents Bootstrap register map of a `GigE` device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Bootstrap {}

impl Bootstrap {
    pub fn new() -> Self {
        Self {}
    }

    pub fn version<Ctrl: DeviceControl + ?Sized>(
        self,
        device: &mut Ctrl,
    ) -> ControlResult<Version> {
        let version: u32 = read_reg(device, bootstrap::VERSION)?;
        let major = version >> 16;
        let minor = version & 0xffff;
        Ok(Version::new(major as u64, minor as u64, 0))
    }

    pub fn device_mode<Ctrl: DeviceControl + ?Sized>(
        self,
        device: &mut Ctrl,
    ) -> ControlResult<DeviceMode> {
        let mode = DeviceMode::from_raw(read_reg(device, bootstrap::DEVICE_MODE)?);
        Ok(mode)
    }

    pub fn mac_addr<Ctrl: DeviceControl + ?Sized>(
        self,
        device: &mut Ctrl,
    ) -> ControlResult<[u8; 6]> {
        let high: [u8; 4] = read_reg(device, bootstrap::DEVICE_MAC_ADDRESS_HIGH_0)?;
        let low: [u8; 4] = read_reg(device, bootstrap::DEVICE_MAC_ADDRESS_LOW_0)?;
        let mut result = [0; 6];
        result.copy_from_slice(&high[2..]);
        result.copy_from_slice(&low);

        Ok(result)
    }

    pub fn nic_capability<Ctrl: DeviceControl + ?Sized>(
        self,
        device: &mut Ctrl,
    ) -> ControlResult<NicCapability> {
        let cap =
            NicCapability::from_raw(read_reg(device, bootstrap::NETWORK_INTERFACE_CAPABILITY_0)?);
        Ok(cap)
    }

    pub fn nic_configuration<Ctrl: DeviceControl + ?Sized>(
        self,
        device: &mut Ctrl,
    ) -> ControlResult<NicConfiguration> {
        let cfg = NicConfiguration::from_raw(read_reg(
            device,
            bootstrap::NETWORK_INTERFACE_CONFIGURATION_0,
        )?);
        Ok(cfg)
    }

    pub fn set_nic_configuration<Ctrl: DeviceControl + ?Sized>(
        self,
        device: &mut Ctrl,
        config: NicConfiguration,
    ) -> ControlResult<()> {
        write_reg(
            device,
            bootstrap::NETWORK_INTERFACE_CONFIGURATION_0,
            config.as_raw(),
        )
    }

    pub fn ip_addr<Ctrl: DeviceControl + ?Sized>(
        self,
        device: &mut Ctrl,
    ) -> ControlResult<Ipv4Addr> {
        read_reg(device, bootstrap::CURRENT_IP_ADDRESS_0)
    }

    pub fn subnet_mask<Ctrl: DeviceControl + ?Sized>(
        self,
        device: &mut Ctrl,
    ) -> ControlResult<[u8; 4]> {
        read_reg(device, bootstrap::CURRENT_SUBNET_MASK_0)
    }

    pub fn default_gateway<Ctrl: DeviceControl + ?Sized>(
        self,
        device: &mut Ctrl,
    ) -> ControlResult<Ipv4Addr> {
        read_reg(device, bootstrap::CURRENT_DEFAULT_GATEWAY_0)
    }

    pub fn manufacturer_name<Ctrl: DeviceControl + ?Sized>(
        self,
        device: &mut Ctrl,
    ) -> ControlResult<String> {
        const LEN: usize = bootstrap::MANUFACTURER_NAME.1 as usize;
        let name: StaticString<LEN> = read_mem(device, bootstrap::MANUFACTURER_NAME)?;
        Ok(name.into_string())
    }

    pub fn model_name<Ctrl: DeviceControl + ?Sized>(
        self,
        device: &mut Ctrl,
    ) -> ControlResult<String> {
        const LEN: usize = bootstrap::MODEL_NAME.1 as usize;
        let name: StaticString<LEN> = read_mem(device, bootstrap::MODEL_NAME)?;
        Ok(name.into_string())
    }

    pub fn manufacturer_info<Ctrl: DeviceControl + ?Sized>(
        self,
        device: &mut Ctrl,
    ) -> ControlResult<String> {
        const LEN: usize = bootstrap::MANUFACTURER_INFO.1 as usize;
        let name: StaticString<LEN> = read_mem(device, bootstrap::MANUFACTURER_INFO)?;
        Ok(name.into_string())
    }

    pub fn serial_number<Ctrl: DeviceControl + ?Sized>(
        self,
        device: &mut Ctrl,
    ) -> ControlResult<String> {
        const LEN: usize = bootstrap::SERIAL_NUMBER.1 as usize;
        let name: StaticString<LEN> = read_mem(device, bootstrap::SERIAL_NUMBER)?;
        Ok(name.into_string())
    }

    pub fn user_defined_name<Ctrl: DeviceControl + ?Sized>(
        self,
        device: &mut Ctrl,
    ) -> ControlResult<String> {
        const LEN: usize = bootstrap::USER_DEFINED_NAME.1 as usize;
        let name: StaticString<LEN> = read_mem(device, bootstrap::USER_DEFINED_NAME)?;
        Ok(name.into_string())
    }

    pub fn set_user_defined_name<Ctrl: DeviceControl + ?Sized>(
        self,
        device: &mut Ctrl,
        name: &str,
    ) -> ControlResult<()> {
        const LEN: usize = bootstrap::USER_DEFINED_NAME.1 as usize;
        let name: StaticString<LEN> = StaticString::from_string(name.to_string())?;
        write_mem(device, bootstrap::USER_DEFINED_NAME, name)
    }

    pub fn first_url<Ctrl: DeviceControl + ?Sized>(
        self,
        device: &mut Ctrl,
    ) -> ControlResult<String> {
        const LEN: usize = bootstrap::FIRST_URL.1 as usize;
        let url: StaticString<LEN> = read_mem(device, bootstrap::FIRST_URL)?;
        Ok(url.into_string())
    }

    pub fn second_url<Ctrl: DeviceControl + ?Sized>(
        self,
        device: &mut Ctrl,
    ) -> ControlResult<String> {
        const LEN: usize = bootstrap::SECOND_URL.1 as usize;
        let url: StaticString<LEN> = read_mem(device, bootstrap::SECOND_URL)?;
        Ok(url.into_string())
    }

    pub fn number_of_nic<Ctrl: DeviceControl + ?Sized>(
        self,
        device: &mut Ctrl,
    ) -> ControlResult<u32> {
        read_reg(device, bootstrap::NUMBER_OF_NETWORK_INTERFACES)
    }

    pub fn number_of_message_channel<Ctrl: DeviceControl + ?Sized>(
        self,
        device: &mut Ctrl,
    ) -> ControlResult<u32> {
        read_reg(device, bootstrap::NUMBER_OF_MESSAGE_CHANNELS)
    }

    pub fn number_of_stream_channel<Ctrl: DeviceControl + ?Sized>(
        self,
        device: &mut Ctrl,
    ) -> ControlResult<u32> {
        read_reg(device, bootstrap::NUMBER_OF_STREAM_CHANNELS)
    }

    pub fn gvcp_capability<Ctrl: DeviceControl + ?Sized>(
        self,
        device: &mut Ctrl,
    ) -> ControlResult<GvcpCapability> {
        let capability = GvcpCapability::from_raw(read_reg(device, bootstrap::GVCP_CAPABILITY)?);
        Ok(capability)
    }

    pub fn gvsp_capability<Ctrl: DeviceControl + ?Sized>(
        self,
        device: &mut Ctrl,
    ) -> ControlResult<GvspCapability> {
        let cap = GvspCapability::from_raw(read_reg(device, bootstrap::GVSP_CAPABILITY)?);
        Ok(cap)
    }

    pub fn message_channel_capability<Ctrl: DeviceControl + ?Sized>(
        self,
        device: &mut Ctrl,
    ) -> ControlResult<MessageChannelCapability> {
        let cap = MessageChannelCapability::from_raw(read_reg(
            device,
            bootstrap::MESSAGE_CHANNEL_CAPABILITY,
        )?);
        Ok(cap)
    }

    pub fn heartbeat_timeout<Ctrl: DeviceControl + ?Sized>(
        self,
        device: &mut Ctrl,
    ) -> ControlResult<time::Duration> {
        let time_raw: u32 = read_reg(device, bootstrap::HEARTBEAT_TIMEOUT)?;
        let time = time::Duration::from_millis(time_raw as u64);
        Ok(time)
    }

    pub fn set_heartbeat_timeout<Ctrl: DeviceControl + ?Sized>(
        self,
        device: &mut Ctrl,
        value: time::Duration,
    ) -> ControlResult<()> {
        let time_raw: u32 =
            unwrap_or_log!(value
                .as_millis()
                .try_into()
                .map_err(|_| ControlError::InvalidData(
                    format!(
                        "too long time is specified for heartbeat timeout: {:?}",
                        value
                    )
                    .into()
                )));

        write_reg(device, bootstrap::HEARTBEAT_TIMEOUT, time_raw)
    }

    pub fn pending_timeout<Ctrl: DeviceControl + ?Sized>(
        self,
        device: &mut Ctrl,
    ) -> ControlResult<time::Duration> {
        let time_raw: u32 = read_reg(device, bootstrap::PENDING_TIMEOUT)?;
        let time = time::Duration::from_millis(time_raw as u64);
        Ok(time)
    }

    pub fn control_channel_priviledge<Ctrl: DeviceControl + ?Sized>(
        self,
        device: &mut Ctrl,
    ) -> ControlResult<ControlChannelPriviledge> {
        let priviledge = ControlChannelPriviledge::from_raw(read_reg(
            device,
            bootstrap::CONTROL_CHANNEL_PRIVILEDGE,
        )?);
        Ok(priviledge)
    }

    pub fn set_control_channel_priviledge<Ctrl: DeviceControl + ?Sized>(
        self,
        device: &mut Ctrl,
        priviledge: ControlChannelPriviledge,
    ) -> ControlResult<()> {
        write_reg(
            device,
            bootstrap::CONTROL_CHANNEL_PRIVILEDGE,
            priviledge.as_raw(),
        )
    }

    pub fn manifest_header<Ctrl: DeviceControl + ?Sized>(
        self,
        device: &mut Ctrl,
    ) -> ControlResult<ManifestHeader> {
        let header = ManifestHeader::new(device)?;
        Ok(header)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamRegister {
    index: u32,
}

impl StreamRegister {
    pub fn new(index: u32) -> Self {
        Self { index }
    }

    pub fn channel_port<Ctrl: DeviceControl + ?Sized>(
        self,
        device: &mut Ctrl,
    ) -> ControlResult<StreamChannelPort> {
        let register = self.register(stream::STREAM_CHANNEL_PORT);
        let stream_channel_port = StreamChannelPort::from_raw(read_reg(device, register)?);

        Ok(stream_channel_port)
    }

    pub fn set_channel_port<Ctrl: DeviceControl + ?Sized>(
        self,
        device: &mut Ctrl,
        port: StreamChannelPort,
    ) -> ControlResult<()> {
        let register = self.register(stream::STREAM_CHANNEL_PORT);
        write_reg(device, register, port.as_raw())
    }

    pub fn packet_size<Ctrl: DeviceControl + ?Sized>(
        self,
        device: &mut Ctrl,
    ) -> ControlResult<StreamPacketSize> {
        let register = self.register(stream::STREAM_CHANNEL_PACKET_SIZE);
        let packet_size = StreamPacketSize::from_raw(read_reg(device, register)?);

        Ok(packet_size)
    }

    pub fn set_packet_size<Ctrl: DeviceControl + ?Sized>(
        self,
        device: &mut Ctrl,
        size: StreamPacketSize,
    ) -> ControlResult<()> {
        let register = self.register(stream::STREAM_CHANNEL_PACKET_SIZE);
        write_reg(device, register, size.as_raw())
    }

    pub fn destination_address<Ctrl: DeviceControl + ?Sized>(
        self,
        device: &mut Ctrl,
    ) -> ControlResult<Ipv4Addr> {
        let register = self.register(stream::STREAM_CHANNEL_DESTINATION_ADDRESS);
        read_reg(device, register)
    }

    pub fn set_destination_address<Ctrl: DeviceControl + ?Sized>(
        self,
        device: &mut Ctrl,
        address: Ipv4Addr,
    ) -> ControlResult<()> {
        let register = self.register(stream::STREAM_CHANNEL_DESTINATION_ADDRESS);
        write_reg(device, register, address)
    }

    fn register(self, register: (u32, u16)) -> (u32, u16) {
        let base = stream::base_address(self.index);
        let addr = base + register.0;
        (addr, register.1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManifestHeader(register_map::ManifestHeader);

impl ManifestHeader {
    fn new<Ctrl: DeviceControl + ?Sized>(device: &mut Ctrl) -> ControlResult<Self> {
        let inner =
            register_map::ManifestHeader::from_raw(read_reg(device, bootstrap::MANIFEST_HEADER)?);
        Ok(Self(inner))
    }

    pub fn entries<Ctrl: DeviceControl + ?Sized>(
        self,
        device: &mut Ctrl,
    ) -> impl Iterator<Item = ControlResult<ManifestEntry>> + '_ {
        (0..self.0.entry_num()).into_iter().map(move |id| {
            let entry_reg = bootstrap::manifest_entry(id);
            let inner = register_map::ManifestEntry::from_raw(read_reg(device, entry_reg)?);
            Ok(ManifestEntry(inner))
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManifestEntry(register_map::ManifestEntry);

impl ManifestEntry {
    pub fn xml_file_version(self) -> Version {
        self.0.xml_file_version()
    }

    pub fn schema_version(self) -> Version {
        self.0.schema_version()
    }

    pub fn url_string<Ctrl: DeviceControl + ?Sized>(
        self,
        device: &mut Ctrl,
    ) -> ControlResult<String> {
        const LEN: usize = bootstrap::FIRST_URL.1 as usize;
        let url_address = self.0.url_register().0;
        let url_string: StaticString<LEN> = read_mem(device, (url_address, LEN as u16))?;
        Ok(url_string.into_string())
    }
}

fn read_reg<Ctrl, T>(device: &mut Ctrl, register: (u32, u16)) -> ControlResult<T>
where
    Ctrl: DeviceControl + ?Sized,
    T: BytesConvertible,
{
    let data = device.read_reg(register.0 as u64)?;
    data.as_ref().read_bytes_be().map_err(Into::into)
}

fn write_reg<Ctrl, T>(device: &mut Ctrl, register: (u32, u16), data: T) -> ControlResult<()>
where
    Ctrl: DeviceControl + ?Sized,
    T: BytesConvertible,
{
    let mut buf = [0; 4];
    buf.as_mut().write_bytes_be(data)?;
    device.write_reg(register.0 as u64, buf)
}

fn read_mem<Ctrl, T>(device: &mut Ctrl, register: (u32, u16)) -> ControlResult<T>
where
    Ctrl: DeviceControl + ?Sized,
    T: BytesConvertible,
{
    let mut buf = vec![0; register.0 as usize];
    device.read(register.0 as u64, &mut buf)?;
    buf.as_slice().read_bytes_be().map_err(Into::into)
}

fn write_mem<Ctrl, T>(device: &mut Ctrl, register: (u32, u16), data: T) -> ControlResult<()>
where
    Ctrl: DeviceControl + ?Sized,
    T: BytesConvertible,
{
    let mut buf = vec![0; register.0 as usize];
    buf.write_bytes_be(data)?;
    device.write(register.0 as u64, &buf)
}
