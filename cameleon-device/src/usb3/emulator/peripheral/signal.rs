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
    EventData {
        event_id: u16,
        data: Vec<u8>,
        request_id: u16,
    },
    UpdateTimestamp(u64),
    Enable,
    Disable(oneshot::Sender<()>),
    Shutdown,
    // TODO: Multievent support.
}

pub(super) enum StreamSignal {
    // TODO: Its' better to send strem protocol with enable signal.
    Enable,
    Disable(oneshot::Sender<()>),
    Shutdown,
}
