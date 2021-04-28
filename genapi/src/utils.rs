use super::{
    interface::{IBoolean, IInteger},
    store::{CacheStore, NodeId, NodeStore, ValueStore},
    Device, GenApiError, GenApiResult, ValueCtxt,
};

pub(super) fn bool_from_id<T: ValueStore, U: CacheStore>(
    node_id: NodeId,
    device: &mut impl Device,
    store: &impl NodeStore,
    cx: &mut ValueCtxt<T, U>,
) -> GenApiResult<bool> {
    if let Some(node) = node_id.as_iboolean_kind(store) {
        node.value(device, store, cx)
    } else if let Some(node) = node_id.as_iinteger_kind(store) {
        Ok(node.value(device, store, cx)? == 1)
    } else {
        Err(GenApiError::InvalidNode(
            "the node doesn't implement `IInteger` nor `IBoolean",
        ))
    }
}
