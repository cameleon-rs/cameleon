/// This packet is sent from a host.
#[derive(Debug)]
pub(crate) struct FakeReqPacket {
    pub(crate) iface: IfaceKind,
    pub(crate) req_kind: FakeReqKind,
}

#[derive(Debug)]
pub(crate) enum FakeReqKind {
    Send(Vec<u8>),
    Recv,
    SetHalt,
    ClearHalt,
}

impl FakeReqPacket {
    pub(crate) fn new(iface: IfaceKind, req_kind: FakeReqKind) -> Self {
        Self { iface, req_kind }
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
            Self::SetHalt => true,
            _ => false,
        }
    }
}

/// This packet is sent from a device.
#[derive(Debug)]
pub(crate) struct FakeAckPacket {
    pub(crate) iface: IfaceKind,
    pub(crate) ack_kind: FakeAckKind,
}

impl FakeAckPacket {
    pub(super) fn new(iface: IfaceKind, ack_kind: FakeAckKind) -> Self {
        Self { iface, ack_kind }
    }
}

#[derive(Debug)]
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
