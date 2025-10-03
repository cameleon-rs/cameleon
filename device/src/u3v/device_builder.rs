/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::{num::NonZeroU8, time::Duration};

use cameleon_impl::bytes_io::ReadBytes;
use log::warn;
use nusb::{
    descriptors::{self, ConfigurationDescriptor, InterfaceDescriptor, TransferType},
    transfer::Direction,
    DeviceInfo as NusbDeviceInfo, MaybeFuture,
};

use crate::u3v::{BusSpeed, DeviceInfo, Error, Result};

use super::{
    channel::{ControlIfaceInfo, ReceiveIfaceInfo},
    device::Device,
};

const MISCELLANEOUS_CLASS: u8 = 0xEF;
const DEVICE_SUBCLASS: u8 = 0x02;
const DEVICE_PROTOCOL: u8 = 0x01;

const IAD_DESC_TYPE: u8 = 0x0B;
const IAD_FUNCTION_PROTOCOL: u8 = 0x00;

const USB3V_SUBCLASS: u8 = 0x05;

const STRING_TIMEOUT: Duration = Duration::from_secs(1);

pub fn enumerate_devices() -> Result<Vec<Device>> {
    let devices_iter = nusb::list_devices().wait().map_err(Error::from)?;
    let mut devices = Vec::new();

    for info in devices_iter {
        match build_device(info) {
            Ok(Some(device)) => devices.push(device),
            Ok(None) => {}
            Err(err) => warn!("failed to initialize u3v device: {err}"),
        }
    }

    Ok(devices)
}

fn build_device(info: NusbDeviceInfo) -> Result<Option<Device>> {
    if info.class() != MISCELLANEOUS_CLASS
        || info.subclass() != DEVICE_SUBCLASS
        || info.protocol() != DEVICE_PROTOCOL
    {
        return Ok(None);
    }

    let device = info.open().wait().map_err(Error::from)?;

    let Some((iad, config_desc)) = find_u3v_iad(&device)? else {
        return Ok(None);
    };

    ensure_configuration(&device, config_desc.configuration_value())?;

    let ctrl_iface = find_control_interface(&config_desc, &iad)?;
    let ctrl_iface_info = ControlIfaceInfo::new(&ctrl_iface)?;

    let device_info_desc = DeviceInfoDescriptor::from_bytes(ctrl_iface.descriptors().as_bytes())?;
    let device_info = device_info_desc.interpret(&device)?;

    let receive_ifaces = collect_receive_ifaces(&config_desc, &iad)?;

    let mut event_iface_info = None;
    let mut stream_iface_info = None;
    for (iface_info, kind) in receive_ifaces {
        match kind {
            ReceiveIfaceKind::Event => event_iface_info = Some(iface_info),
            ReceiveIfaceKind::Stream => stream_iface_info = Some(iface_info),
        }
    }

    Ok(Some(Device::new(
        device,
        ctrl_iface_info,
        event_iface_info,
        stream_iface_info,
        device_info,
    )))
}

fn find_u3v_iad<'a>(
    device: &'a nusb::Device,
) -> Result<Option<(Iad, ConfigurationDescriptor<'a>)>> {
    for config in device.configurations() {
        if let Some(iad) = find_u3v_iad_in_config_desc(&config) {
            return Ok(Some((iad, config)));
        }
    }

    Ok(None)
}

fn find_u3v_iad_in_config_desc(desc: &ConfigurationDescriptor<'_>) -> Option<Iad> {
    if let Some(iad) = Iad::from_bytes(desc.descriptors().as_bytes()) {
        if Iad::is_u3v(&iad) {
            return Some(iad);
        }
    }

    for iface in desc.interfaces() {
        for alt in iface.alt_settings() {
            if let Some(iad) = Iad::from_bytes(alt.as_bytes()) {
                if Iad::is_u3v(&iad) {
                    return Some(iad);
                }
            }

            for ep in alt.endpoints() {
                if let Some(iad) = Iad::from_bytes(ep.as_bytes()) {
                    if Iad::is_u3v(&iad) {
                        return Some(iad);
                    }
                }
            }
        }
    }

    None
}

fn find_control_interface<'a>(
    config: &ConfigurationDescriptor<'a>,
    iad: &Iad,
) -> Result<InterfaceDescriptor<'a>> {
    for iface in config.interfaces() {
        if iface.interface_number() == iad.first_interface {
            if let Some(desc) = iface.alt_settings().find(|d| d.alternate_setting() == 0) {
                return Ok(desc);
            }
        }
    }

    Err(Error::InvalidDevice)
}

fn collect_receive_ifaces<'a>(
    config: &ConfigurationDescriptor<'a>,
    iad: &Iad,
) -> Result<Vec<(ReceiveIfaceInfo, ReceiveIfaceKind)>> {
    let first = iad.first_interface;
    let count = iad.interface_count;

    let mut result = Vec::new();
    for iface in config.interfaces() {
        let iface_number = iface.interface_number();
        if iface_number < first || iface_number >= first + count {
            continue;
        }
        if iface_number == first {
            continue;
        }

        for desc in iface.alt_settings() {
            if desc.alternate_setting() != 0 {
                continue;
            }
            if let Some(entry) = ReceiveIfaceInfo::new(&desc) {
                result.push(entry);
            }
        }
    }

    if result.len() > 2 {
        return Err(Error::InvalidDevice);
    }

    Ok(result)
}

fn ensure_configuration(device: &nusb::Device, desired: u8) -> Result<()> {
    match device.active_configuration() {
        Ok(active) if active.configuration_value() == desired => Ok(()),
        Ok(_) | Err(_) => device
            .set_configuration(desired)
            .wait()
            .map_err(Error::from),
    }
}

#[derive(PartialEq)]
enum ReceiveIfaceKind {
    Stream,
    Event,
}

/// Interface Association Descriptor.
struct Iad {
    _length: u8,
    _descriptor_type: u8,
    first_interface: u8,
    interface_count: u8,
    function_class: u8,
    function_subclass: u8,
    function_protocol: u8,
    _function: u8,
}

impl Iad {
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut read = 0;
        let len = bytes.len();

        while read + 8 <= len {
            let desc_length = bytes[read];
            if desc_length == 0 {
                break;
            } else if desc_length == 1 {
                read += desc_length as usize;
                continue;
            }

            if read + desc_length as usize > len {
                break;
            }

            let descriptor_type = bytes[read + 1];
            if descriptor_type != IAD_DESC_TYPE {
                read += desc_length as usize;
                continue;
            }

            let first_interface = bytes[read + 2];
            let interface_count = bytes[read + 3];
            let function_class = bytes[read + 4];
            let function_subclass = bytes[read + 5];
            let function_protocol = bytes[read + 6];
            let function = bytes[read + 7];

            return Some(Self {
                _length: desc_length,
                _descriptor_type: descriptor_type,
                first_interface,
                interface_count,
                function_class,
                function_subclass,
                function_protocol,
                _function: function,
            });
        }

        None
    }

    fn is_u3v(&self) -> bool {
        self.function_class == MISCELLANEOUS_CLASS
            && self.function_subclass == USB3V_SUBCLASS
            && self.function_protocol == IAD_FUNCTION_PROTOCOL
    }
}

impl ControlIfaceInfo {
    fn new(iface: &InterfaceDescriptor<'_>) -> Result<Self> {
        if iface.class() != MISCELLANEOUS_CLASS
            || iface.subclass() != USB3V_SUBCLASS
            || iface.protocol() != 0
        {
            return Err(Error::InvalidDevice);
        }

        let endpoints: Vec<_> = iface.endpoints().collect();
        if endpoints.len() != 2 {
            return Err(Error::InvalidDevice);
        }

        let ep_in = endpoints
            .iter()
            .find(|ep| ep.direction() == Direction::In)
            .ok_or(Error::InvalidDevice)?;
        let ep_out = endpoints
            .iter()
            .find(|ep| ep.direction() == Direction::Out)
            .ok_or(Error::InvalidDevice)?;

        if ep_in.transfer_type() != TransferType::Bulk
            || ep_out.transfer_type() != TransferType::Bulk
        {
            return Err(Error::InvalidDevice);
        }

        Ok(Self {
            iface_number: iface.interface_number(),
            bulk_in_ep: ep_in.address(),
            bulk_out_ep: ep_out.address(),
        })
    }
}

impl ReceiveIfaceInfo {
    fn new(iface: &InterfaceDescriptor<'_>) -> Option<(Self, ReceiveIfaceKind)> {
        if iface.class() != MISCELLANEOUS_CLASS || iface.subclass() != USB3V_SUBCLASS {
            return None;
        }

        let kind = match iface.protocol() {
            0x01 => ReceiveIfaceKind::Event,
            0x02 => ReceiveIfaceKind::Stream,
            _ => return None,
        };

        if iface.num_endpoints() != 1 {
            return None;
        }

        let endpoint = iface.endpoints().next().unwrap();
        if endpoint.transfer_type() != TransferType::Bulk || endpoint.direction() != Direction::In {
            return None;
        }

        let info = ReceiveIfaceInfo {
            iface_number: iface.interface_number(),
            alt_setting: iface.alternate_setting(),
            bulk_in_ep: endpoint.address(),
        };

        Some((info, kind))
    }
}

struct DeviceInfoDescriptor {
    _length: u8,
    _descriptor_type: u8,
    _descriptor_subtype: u8,
    gencp_version_major: u16,
    gencp_version_minor: u16,
    u3v_version_major: u16,
    u3v_version_minor: u16,
    guid_idx: u8,
    vendor_name_idx: u8,
    model_name_idx: u8,
    family_name_idx: u8,
    device_version_idx: u8,
    manufacturer_info_idx: u8,
    serial_number_idx: u8,
    user_defined_name_idx: u8,
    supported_speed_mask: u8,
}

impl DeviceInfoDescriptor {
    const MINIMUM_DESC_LENGTH: u8 = 20;
    const DESCRIPTOR_TYPE: u8 = 0x24;
    const DESCRIPTOR_SUBTYPE: u8 = 0x1;

    fn from_bytes(mut bytes: &[u8]) -> Result<Self> {
        if bytes.len() < Self::MINIMUM_DESC_LENGTH as usize {
            return Err(Error::InvalidDevice);
        }

        let length: u8 = bytes.read_bytes_le()?;
        let descriptor_type = bytes.read_bytes_le()?;
        let descriptor_subtype = bytes.read_bytes_le()?;

        if length < Self::MINIMUM_DESC_LENGTH
            || descriptor_type != Self::DESCRIPTOR_TYPE
            || descriptor_subtype != Self::DESCRIPTOR_SUBTYPE
        {
            return Err(Error::InvalidDevice);
        }

        let gencp_version_minor = bytes.read_bytes_le()?;
        let gencp_version_major = bytes.read_bytes_le()?;
        let u3v_version_minor = bytes.read_bytes_le()?;
        let u3v_version_major = bytes.read_bytes_le()?;
        let guid_idx = bytes.read_bytes_le()?;
        let vendor_name_idx = bytes.read_bytes_le()?;
        let model_name_idx = bytes.read_bytes_le()?;
        let family_name_idx = bytes.read_bytes_le()?;
        let device_version_idx = bytes.read_bytes_le()?;
        let manufacturer_info_idx = bytes.read_bytes_le()?;
        let serial_number_idx = bytes.read_bytes_le()?;
        let user_defined_name_idx = bytes.read_bytes_le()?;
        let supported_speed_mask = bytes.read_bytes_le()?;

        Ok(Self {
            _length: length,
            _descriptor_type: descriptor_type,
            _descriptor_subtype: descriptor_subtype,
            gencp_version_major,
            gencp_version_minor,
            u3v_version_major,
            u3v_version_minor,
            guid_idx,
            vendor_name_idx,
            model_name_idx,
            family_name_idx,
            device_version_idx,
            manufacturer_info_idx,
            serial_number_idx,
            user_defined_name_idx,
            supported_speed_mask,
        })
    }

    fn interpret(&self, device: &nusb::Device) -> Result<DeviceInfo> {
        use descriptors::language_id::US_ENGLISH;

        let languages: Vec<_> = device
            .get_string_descriptor_supported_languages(STRING_TIMEOUT)
            .wait()
            .map_err(Error::from)?
            .collect();
        let language_id = languages.first().copied().unwrap_or(US_ENGLISH);

        let guid = read_string(device, self.guid_idx, language_id)?;
        let vendor_name = read_string(device, self.vendor_name_idx, language_id)?;
        let model_name = read_string(device, self.model_name_idx, language_id)?;
        let family_name = optional_string(device, self.family_name_idx, language_id)?;
        let device_version = read_string(device, self.device_version_idx, language_id)?;
        let manufacturer_info = read_string(device, self.manufacturer_info_idx, language_id)?;
        let serial_number = read_string(device, self.serial_number_idx, language_id)?;
        let user_defined_name = optional_string(device, self.user_defined_name_idx, language_id)?;

        let gencp_version = semver::Version::new(
            self.gencp_version_major.into(),
            self.gencp_version_minor.into(),
            0,
        );
        let u3v_version = semver::Version::new(
            self.u3v_version_major.into(),
            self.u3v_version_minor.into(),
            0,
        );

        let supported_speed = if (self.supported_speed_mask >> 4) & 0b1 == 1 {
            BusSpeed::SuperSpeedPlus
        } else if (self.supported_speed_mask >> 3) & 0b1 == 1 {
            BusSpeed::SuperSpeed
        } else if (self.supported_speed_mask >> 2) & 0b1 == 1 {
            BusSpeed::HighSpeed
        } else if (self.supported_speed_mask >> 1) & 0b1 == 1 {
            BusSpeed::FullSpeed
        } else if self.supported_speed_mask & 0b1 == 1 {
            BusSpeed::LowSpeed
        } else {
            return Err(Error::InvalidDevice);
        };

        Ok(DeviceInfo {
            gencp_version,
            u3v_version,
            guid,
            vendor_name,
            model_name,
            family_name,
            device_version,
            manufacturer_info,
            serial_number,
            user_defined_name,
            supported_speed,
        })
    }
}

fn read_string(device: &nusb::Device, index: u8, language_id: u16) -> Result<String> {
    optional_string(device, index, language_id)?.ok_or(Error::InvalidDevice)
}

fn optional_string(device: &nusb::Device, index: u8, language_id: u16) -> Result<Option<String>> {
    if index == 0 {
        return Ok(None);
    }

    let index = NonZeroU8::new(index).ok_or(Error::InvalidDevice)?;
    let value = device
        .get_string_descriptor(index, language_id, STRING_TIMEOUT)
        .wait()
        .map_err(Error::from)?;
    Ok(Some(value))
}
