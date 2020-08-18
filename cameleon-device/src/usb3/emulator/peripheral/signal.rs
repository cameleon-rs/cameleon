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
    EventData(Vec<u8>),
    UpdateTimestamp(u32),
    Start,
    Pause(oneshot::Sender<()>),
    Shutdown(oneshot::Sender<()>),
}

pub(super) enum StreamSignal {
    Start,
    Pause(oneshot::Sender<()>),
    Shutdown(oneshot::Sender<()>),
    // TODO: Stream module property(i.e. pixel format or stream protocol) will also be managed through this signal.
}
