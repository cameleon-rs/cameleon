use futures::channel::oneshot;

use super::fake_protocol::IfaceKind;

pub(super) enum CtrlSignal {
    SendDataReq(Vec<u8>),

    SetHalt {
        iface: IfaceKind,
        completed: oneshot::Sender<()>,
    },

    Shutdown(oneshot::Sender<()>),
}

pub(super) enum EventSignal {
    _EventData {
        event_id: u16,
        data: Vec<u8>,
        request_id: u16,
    },
    UpdateTimestamp(u64),
    _Enable,
    Disable(oneshot::Sender<()>),
    Shutdown(oneshot::Sender<()>),
    // TODO: Multievent support.
}

pub(super) enum StreamSignal {
    // TODO: It's better to send strem protocol with enable signal.
    _Enable,
    Disable(oneshot::Sender<()>),
    Shutdown(oneshot::Sender<()>),
}
