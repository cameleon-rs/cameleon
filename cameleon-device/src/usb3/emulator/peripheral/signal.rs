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
    Pause(oneshot::Sender<()>),
    Shutdown(oneshot::Sender<()>),
}

pub(super) enum StreamManagementSignal {
    Start,
    Pause(oneshot::Sender<()>),
    Shutdown(oneshot::Sender<()>),
    // TODO: Stream module property(i.e. pixel format or stream protocol) will also be managed through this signal.
}
