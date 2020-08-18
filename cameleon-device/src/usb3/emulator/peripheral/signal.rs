use futures::channel::oneshot;

use super::fake_protocol::IfaceKind;

pub(super) enum CtrlSignal {
    SendDataReq(Vec<u8>),

    SetHalt {
        iface: IfaceKind,
        completed: oneshot::Sender<()>,
    },

    Shutdown,
}

pub(super) enum EventSignal {
    EventData(u16),
    UpdateTimestamp(u32),
    Enable,
    Disable(oneshot::Sender<()>),
    Shutdown,
    // TODO: Multievent support.
}

pub(super) enum StreamSignal {
    Enable,
    Disable(oneshot::Sender<()>),
    Shutdown,
    // TODO: Stream module property(e.g. pixel format or stream protocol) will also be managed through this signal.
}
