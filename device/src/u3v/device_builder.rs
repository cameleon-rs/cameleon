/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use cameleon_impl::bytes_io::ReadBytes;
use semver::Version;

use crate::u3v::{BusSpeed, DeviceInfo, Error, Result};

use super::{
    channel::{ControlIfaceInfo, ReceiveIfaceInfo},
    device::{Device, RusbDevice, RusbDeviceHandle},
};

const MISCELLANEOUS_CLASS: u8 = 0xEF;

const DEVICE_SUBCLASS: u8 = 0x02;
const DEVICE_PROTOCOL: u8 = 0x01;

const IAD_DESC_TYPE: u8 = 0x0B;
const IAD_FUNCTION_PROTOCOL: u8 = 0x00;

const USB3V_SUBCLASS: u8 = 0x05;

pub fn enumerate_devices() -> Result<Vec<Device>> {
    let rusb_device_list = rusb::DeviceList::new()?;
    let builders = rusb_device_list
        .iter()
        .filter_map(|dev| DeviceBuilder::new(dev).ok().flatten());

    Ok(builders
        .filter_map(|builder| builder.build().ok())
        .collect())
}

struct DeviceBuilder {
    device: RusbDevice,
    u3v_iad: Iad,
    config_desc: rusb::ConfigDescriptor,
}

impl DeviceBuilder {
    fn new(device: RusbDevice) -> Result<Option<Self>> {
        let device_desc = device.device_descriptor()?;

        if device_desc.class_code() == MISCELLANEOUS_CLASS
            && device_desc.sub_class_code() == DEVICE_SUBCLASS
            && device_desc.protocol_code() == DEVICE_PROTOCOL
        {
            if let Some((iad, conf_desc)) = Self::find_u3v_iad(&device, &device_desc)? {
                return Ok(Some(Self {
                    device,
                    u3v_iad: iad,
                    config_desc: conf_desc,
                }));
            }
        }

        Ok(None)
    }

    fn build(self) -> Result<Device> {
        // TODO: Log it when device is broken or invalid.
        let mut dev_channel = self.device.open()?;
        if dev_channel.active_configuration()? != self.config_desc.number() {
            dev_channel.set_active_configuration(self.config_desc.number())?;
        }

        // Skip interfaces while control interface is appeared.
        let mut interfaces = self
            .config_desc
            .interfaces()
            .skip_while(|iface| iface.number() != self.u3v_iad.first_interface);

        // Retrieve control interface information.
        let ctrl_iface = interfaces.next().ok_or(Error::InvalidDevice)?;
        let ctrl_iface_info = ControlIfaceInfo::new(&ctrl_iface)?;

        // Retrieve device information.
        // This information is embedded next to control interface descriptor.
        let ctrl_iface_desc = ctrl_iface
            .descriptors()
            .next()
            .ok_or(Error::InvalidDevice)?;
        let device_info_desc = ctrl_iface_desc.extra().ok_or(Error::InvalidDevice)?;
        let device_info_desc = DeviceInfoDescriptor::from_bytes(device_info_desc)?;
        let device_info = device_info_desc.interpret(&dev_channel)?;

        // Retrieve event and stream interface information if exists.
        let receive_ifaces = interfaces.filter_map(|iface| ReceiveIfaceInfo::new(&iface));
        let mut receive_ifaces: Vec<(ReceiveIfaceInfo, ReceiveIfaceKind)> =
            receive_ifaces.collect();

        if receive_ifaces.len() > 2 {
            return Err(Error::InvalidDevice);
        }

        let (event_iface, stream_iface) = match receive_ifaces.pop() {
            Some((event_iface, ReceiveIfaceKind::Event)) => match receive_ifaces.pop() {
                Some((stream_iface, ReceiveIfaceKind::Stream)) => {
                    (Some(event_iface), Some(stream_iface))
                }
                None => (Some(event_iface), None),
                Some(_) => return Err(Error::InvalidDevice),
            },
            Some((stream_iface, ReceiveIfaceKind::Stream)) => match receive_ifaces.pop() {
                Some((event_iface, ReceiveIfaceKind::Event)) => {
                    (Some(event_iface), Some(stream_iface))
                }
                None => (None, Some(stream_iface)),
                Some(_) => return Err(Error::InvalidDevice),
            },
            None => (None, None),
        };

        Ok(Device::new(
            self.device,
            ctrl_iface_info,
            event_iface,
            stream_iface,
            device_info,
        ))
    }

    fn find_u3v_iad(
        device: &RusbDevice,
        device_desc: &rusb::DeviceDescriptor,
    ) -> Result<Option<(Iad, rusb::ConfigDescriptor)>> {
        let num_config_desc = device_desc.num_configurations();

        for config_index in 0..num_config_desc {
            let config_desc = device.config_descriptor(config_index)?;
            if let Some(u3v_iad) = Self::find_u3v_iad_in_config_desc(&config_desc) {
                return Ok(Some((u3v_iad, config_desc)));
            }
        }

        Ok(None)
    }

    fn find_u3v_iad_in_config_desc(desc: &rusb::ConfigDescriptor) -> Option<Iad> {
        if let Some(extra) = desc.extra() {
            if let Some(iad) = Iad::from_bytes(extra) {
                if Self::is_u3v_iad(&iad) {
                    return Some(iad);
                }
            }
        }

        for iface in desc.interfaces() {
            for if_desc in iface.descriptors() {
                if let Some(u3v_iad) = Self::find_u3v_iad_in_if_desc(&if_desc) {
                    return Some(u3v_iad);
                }
            }
        }

        None
    }

    fn find_u3v_iad_in_if_desc(desc: &rusb::InterfaceDescriptor) -> Option<Iad> {
        if let Some(extra) = desc.extra() {
            if let Some(iad) = Iad::from_bytes(extra) {
                if Self::is_u3v_iad(&iad) {
                    return Some(iad);
                }
            }
        }

        for ep_desc in desc.endpoint_descriptors() {
            if let Some(u3v_iad) = Self::find_u3v_iad_in_ep_desc(&ep_desc) {
                return Some(u3v_iad);
            }
        }

        None
    }

    fn find_u3v_iad_in_ep_desc(desc: &rusb::EndpointDescriptor) -> Option<Iad> {
        if let Some(extra) = desc.extra() {
            if let Some(iad) = Iad::from_bytes(extra) {
                if Self::is_u3v_iad(&iad) {
                    return Some(iad);
                }
            }
        }

        None
    }

    fn is_u3v_iad(iad: &Iad) -> bool {
        iad.function_class == MISCELLANEOUS_CLASS
            && iad.function_subclass == USB3V_SUBCLASS
            && iad.function_protocol == IAD_FUNCTION_PROTOCOL
    }
}

/// Interface Association Descriptor.
#[allow(unused)]
struct Iad {
    length: u8,
    descriptor_type: u8,
    first_interface: u8,
    interface_count: u8,
    function_class: u8,
    function_subclass: u8,
    function_protocol: u8,
    function: u8,
}

impl Iad {
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut read = 0;
        let len = bytes.len();

        while read < len {
            let desc_length = bytes[read];
            if desc_length == 0 {
                break;
            } else if desc_length == 1 {
                read += desc_length as usize;
                continue;
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
                length: desc_length,
                descriptor_type,
                first_interface,
                interface_count,
                function_class,
                function_subclass,
                function_protocol,
                function,
            });
        }

        None
    }
}

struct DeviceInfoDescriptor {
    #[allow(unused)]
    length: u8,
    #[allow(unused)]
    descriptor_type: u8,
    #[allow(unused)]
    descriptor_subtype: u8,
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
            length,
            descriptor_type,
            descriptor_subtype,
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

    fn interpret(&self, channel: &RusbDeviceHandle) -> Result<DeviceInfo> {
        let gencp_version = Version::new(
            self.gencp_version_major.into(),
            self.gencp_version_minor.into(),
            0,
        );

        let u3v_version = Version::new(
            self.u3v_version_major.into(),
            self.u3v_version_minor.into(),
            0,
        );

        let guid = channel.read_string_descriptor_ascii(self.guid_idx)?;
        let vendor_name = channel.read_string_descriptor_ascii(self.vendor_name_idx)?;
        let model_name = channel.read_string_descriptor_ascii(self.model_name_idx)?;
        let family_name = if self.family_name_idx == 0 {
            None
        } else {
            Some(channel.read_string_descriptor_ascii(self.family_name_idx)?)
        };

        let device_version = channel.read_string_descriptor_ascii(self.device_version_idx)?;
        let manufacturer_info = channel.read_string_descriptor_ascii(self.manufacturer_info_idx)?;
        let serial_number = channel.read_string_descriptor_ascii(self.serial_number_idx)?;
        let user_defined_name = if self.user_defined_name_idx == 0 {
            None
        } else {
            Some(channel.read_string_descriptor_ascii(self.user_defined_name_idx)?)
        };
        let supported_speed = if self.supported_speed_mask >> 4 & 0b1 == 1 {
            BusSpeed::SuperSpeedPlus
        } else if self.supported_speed_mask >> 3 & 0b1 == 1 {
            BusSpeed::SuperSpeed
        } else if self.supported_speed_mask >> 2 & 0b1 == 1 {
            BusSpeed::HighSpeed
        } else if self.supported_speed_mask >> 1 & 0b1 == 1 {
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

impl ControlIfaceInfo {
    const CONTROL_IFACE_PROTOCOL: u8 = 0x00;

    fn new(iface: &rusb::Interface) -> Result<Self> {
        let iface_number = iface.number();
        let iface_desc = iface.descriptors().next().ok_or(Error::InvalidDevice)?;

        if iface_desc.class_code() != MISCELLANEOUS_CLASS
            || iface_desc.sub_class_code() != USB3V_SUBCLASS
            || iface_desc.protocol_code() != Self::CONTROL_IFACE_PROTOCOL
        {
            return Err(Error::InvalidDevice);
        }

        let eps: Vec<rusb::EndpointDescriptor> = iface_desc.endpoint_descriptors().collect();
        if eps.len() != 2 {
            return Err(Error::InvalidDevice);
        }
        let ep_in = eps
            .iter()
            .find(|ep| ep.direction() == rusb::Direction::In)
            .ok_or(Error::InvalidDevice)?;
        let ep_out = eps
            .iter()
            .find(|ep| ep.direction() == rusb::Direction::Out)
            .ok_or(Error::InvalidDevice)?;
        if ep_in.transfer_type() != rusb::TransferType::Bulk
            || ep_out.transfer_type() != rusb::TransferType::Bulk
        {
            return Err(Error::InvalidDevice);
        }

        Ok(Self {
            iface_number,
            bulk_in_ep: ep_in.address(),
            bulk_out_ep: ep_out.address(),
        })
    }
}

impl ReceiveIfaceInfo {
    const EVENT_IFACE_PROTOCOL: u8 = 0x01;
    const STREAM_IFACE_PROTOCOL: u8 = 0x02;

    fn new(iface: &rusb::Interface) -> Option<(Self, ReceiveIfaceKind)> {
        let iface_number = iface.number();
        for desc in iface.descriptors() {
            if desc.setting_number() != 0 {
                continue;
            }

            if desc.class_code() != MISCELLANEOUS_CLASS || desc.sub_class_code() != USB3V_SUBCLASS {
                return None;
            }

            let iface_kind = match desc.protocol_code() {
                Self::EVENT_IFACE_PROTOCOL => ReceiveIfaceKind::Event,
                Self::STREAM_IFACE_PROTOCOL => ReceiveIfaceKind::Stream,
                _ => return None,
            };

            if desc.num_endpoints() != 1 {
                return None;
            }
            let ep = desc.endpoint_descriptors().next().unwrap();
            if ep.transfer_type() != rusb::TransferType::Bulk
                || ep.direction() != rusb::Direction::In
            {
                return None;
            }

            let iface_info = ReceiveIfaceInfo {
                iface_number,
                bulk_in_ep: ep.address(),
            };

            return Some((iface_info, iface_kind));
        }

        None
    }
}

#[derive(PartialEq)]
enum ReceiveIfaceKind {
    Stream,
    Event,
}
