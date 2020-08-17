use futures::channel::oneshot;

use super::fake_protocol::IfaceKind;

pub(super) enum CtrlManagementSignal {
    SetHalt {
        iface: IfaceKind,
        completed: oneshot::Sender<()>,
    },

    Shutdown,
}

pub(super) enum EventManagementSignal {
    Start,
    Pause,
    Shutdown,
}

pub(super) enum StreamManagementSignal {
    Start,
    Pause,
    Shutdown,
    // TODO: Stream module property(i.e. pixel format or stream protocol) will also be managed through this signal.
}
