/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub mod ack;
pub mod cmd;
pub mod stream;

pub mod prelude {
    pub use super::cmd::CommandData;
}

use std::io;

use cameleon_impl::bytes_io::ReadBytes;

use crate::gige::{Error, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PacketStatus {
    code: u16,
    kind: StatusKind,
}

impl PacketStatus {
    pub fn is_success(self) -> bool {
        matches!(self.kind, StatusKind::Success | StatusKind::PacketResend)
    }

    pub fn code(self) -> u16 {
        self.code
    }

    pub fn kind(self) -> StatusKind {
        self.kind
    }

    fn parse(cursor: &mut io::Cursor<&[u8]>) -> Result<Self> {
        let code: u16 = cursor.read_bytes_be()?;
        let kind = match code {
            0x0000 => StatusKind::Success,
            0x0100 => StatusKind::PacketResend,
            0x8001 => StatusKind::NotImplemented,
            0x8002 => StatusKind::InvalidParameter,
            0x8003 => StatusKind::InvalidAdderess,
            0x8004 => StatusKind::WriteProtect,
            0x8005 => StatusKind::BadAlignment,
            0x8006 => StatusKind::AccessDenied,
            0x8007 => StatusKind::Busy,
            0x8008 => StatusKind::LocalProblem,
            0x8009 => StatusKind::MessageMismatch,
            0x800a => StatusKind::InvalidProtocol,
            0x800b => StatusKind::NoMessage,
            0x800c => StatusKind::PacketUnavailable,
            0x800d => StatusKind::DataOverrun,
            0x800e => StatusKind::InvalidHeader,
            0x800f => StatusKind::WrongConfig,
            0x8010 => StatusKind::NotYetAvailable,
            0x8011 => StatusKind::CurrentAndPrevPacketRemoved,
            0x8012 => StatusKind::CurrentPacketRemoved,
            0x8013 => StatusKind::NoReferenceTime,
            0x8014 => StatusKind::TemporarilyUnavailable,
            0x8015 => StatusKind::Overflow,
            0x8016 => StatusKind::ActionLate,
            0x8fff => StatusKind::GenericError,
            _ => {
                return Err(Error::InvalidPacket(
                    format! {"invalid gige ack status code {:#X}", code}.into(),
                ));
            }
        };
        Ok(Self { code, kind })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusKind {
    Success,
    PacketResend,
    NotImplemented,
    InvalidParameter,
    InvalidAdderess,
    WriteProtect,
    BadAlignment,
    AccessDenied,
    Busy,
    LocalProblem,
    MessageMismatch,
    InvalidProtocol,
    NoMessage,
    PacketUnavailable,
    DataOverrun,
    InvalidHeader,
    WrongConfig,
    NotYetAvailable,
    CurrentAndPrevPacketRemoved,
    CurrentPacketRemoved,
    NoReferenceTime,
    TemporarilyUnavailable,
    Overflow,
    ActionLate,
    GenericError,
}
