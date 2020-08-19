/// This packet is sent from a host.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct FakeReqPacket {
    pub(crate) iface: IfaceKind,
    pub(crate) kind: FakeReqKind,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum FakeReqKind {
    Send(Vec<u8>),
    Recv,
    SetHalt,
    ClearHalt,
}

impl FakeReqPacket {
    pub(crate) fn new(iface: IfaceKind, kind: FakeReqKind) -> Self {
        Self { iface, kind }
    }

    pub(crate) fn control(kind: FakeReqKind) -> Self {
        Self::new(IfaceKind::Control, kind)
    }

    pub(crate) fn event(kind: FakeReqKind) -> Self {
        Self::new(IfaceKind::Event, kind)
    }

    pub(crate) fn stream(kind: FakeReqKind) -> Self {
        Self::new(IfaceKind::Stream, kind)
    }
}

impl FakeReqKind {
    pub(super) fn is_set_halt(&self) -> bool {
        match self {
            Self::SetHalt => true,
            _ => false,
        }
    }

    pub(super) fn is_clear_halt(&self) -> bool {
        match self {
            Self::ClearHalt => true,
            _ => false,
        }
    }
}

/// This packet is sent from a device.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct FakeAckPacket {
    pub(crate) iface: IfaceKind,
    pub(crate) kind: FakeAckKind,
}

impl FakeAckPacket {
    pub(super) fn new(iface: IfaceKind, kind: FakeAckKind) -> Self {
        Self { iface, kind }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum FakeAckKind {
    SendAck,
    SendNak,
    RecvAck(Vec<u8>),
    RecvNak,
    IfaceHalted,
    SetHaltAck,
    ClearHaltAck,
    BrokenReq,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum IfaceKind {
    Control,
    Event,
    Stream,
}
