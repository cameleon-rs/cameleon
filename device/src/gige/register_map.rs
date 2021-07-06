/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::{CharacterEncoding, Endianness};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceMode(u32);

impl DeviceMode {
    pub fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub fn endianness(self) -> Endianness {
        if self.0 >> 31_u8 == 0 {
            Endianness::LE
        } else {
            Endianness::BE
        }
    }

    pub fn device_class(self) -> DeviceClass {
        let code = (self.0 >> 28_u8) & 0b111;
        match code {
            0 => DeviceClass::Transmitter,
            1 => DeviceClass::Receiver,
            2 => DeviceClass::Transceiver,
            3 => DeviceClass::Peripheral,
            _ => unreachable!(),
        }
    }

    pub fn link_configuration(self) -> LinkConfiguration {
        let code = (self.0 >> 24_u8) & 0b1111;
        match code {
            0 => LinkConfiguration::SingleLink,
            1 => LinkConfiguration::MultipleLink,
            2 => LinkConfiguration::StaticLAG,
            3 => LinkConfiguration::DynamicLAG,
            _ => unreachable!(),
        }
    }

    pub fn char_encoding(self) -> CharacterEncoding {
        let code = self.0 & 0b1111_1111;
        match code {
            1 => CharacterEncoding::Utf8,
            0 | 2 => CharacterEncoding::Ascii,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceClass {
    Transmitter,
    Receiver,
    Transceiver,
    Peripheral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkConfiguration {
    SingleLink,
    MultipleLink,
    StaticLAG,
    DynamicLAG,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NicCapability(u32);

impl NicCapability {
    pub fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub fn is_pause_reception_supported(self) -> bool {
        self.0 >> 31_u8 == 1
    }

    pub fn is_pause_generation_supported(self) -> bool {
        (self.0 >> 31_u8) & 0b1 == 1
    }

    pub fn is_link_local_address_supported(self) -> bool {
        (self.0 >> 2_u8) & 0b1 == 1
    }

    pub fn is_dhcp_supported(self) -> bool {
        (self.0 >> 1_u8) & 0b1 == 1
    }

    pub fn is_force_ip_supported(self) -> bool {
        self.0 & 0b1 == 1
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NicConfiguration(u32);

impl NicConfiguration {
    pub fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub fn is_pause_reception_enabled(self) -> bool {
        self.0 >> 31_u8 == 1
    }

    pub fn is_pause_generation_enabled(self) -> bool {
        (self.0 >> 31_u8) & 0b1 == 1
    }

    pub fn is_link_local_address_enabled(self) -> bool {
        (self.0 >> 2_u8) & 0b1 == 1
    }

    pub fn is_dhcp_enabled(self) -> bool {
        (self.0 >> 1_u8) & 0b1 == 1
    }

    pub fn is_force_ip_enabled(self) -> bool {
        self.0 & 0b1 == 1
    }
}
