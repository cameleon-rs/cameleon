use futures::channel::oneshot;

use super::IfaceKind;

/// Signal sent to control module.
pub(super) enum ControlSignal {
    ReceiveData(Vec<u8>),

    CancelJobs(oneshot::Sender<()>),

    ClearEiRegister,
    ClearSiRegister,

    Shutdown,
}

/// Signal sent to event module.
pub(super) enum EventSignal {
    _EventData {
        event_id: u16,
        data: Vec<u8>,
        request_id: u16,
    },

    UpdateTimestamp(u64),

    _Enable,
    Disable(oneshot::Sender<()>),

    Shutdown,
}

/// Signal sent to stream module.
pub(super) enum StreamSignal {
    _Enable,
    Disable(oneshot::Sender<()>),
    Shutdown,
}

/// Signal sent from each module to interface.
pub(super) enum InterfaceSignal {
    _ToControl(ControlSignal),
    ToEvent(EventSignal),
    _ToStream(StreamSignal),
    Halt(IfaceKind),
}
