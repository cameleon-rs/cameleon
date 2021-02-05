use crate::device::DeviceResult;

use super::register_map::Sirm;

/// Parameters to receive stream packets.
///
/// Both `StreamParams` and [`StreamHandle`] don't check integrity of the paremter. That's up to user.
pub struct StreamParams {
    /// Maximum leader size.
    pub leader_size: u32,

    /// Maximum trailer size.
    pub trailer_size: u32,

    /// Payload transfer size.
    pub payload_size: u32,

    /// Payload transfer count.
    pub payload_count: u32,

    /// Payload transfer final1 size.
    pub payload_final1_size: u32,

    /// Payload transfer final2 size.
    pub payload_final2_size: u32,
}

impl StreamParams {
    /// Constructor of `StreamParams`.
    pub fn new(
        leader_size: u32,
        trailer_size: u32,
        payload_size: u32,
        payload_count: u32,
        payload_final1_size: u32,
        payload_final2_size: u32,
    ) -> Self {
        Self {
            leader_size,
            trailer_size,
            payload_size,
            payload_count,
            payload_final1_size,
            payload_final2_size,
        }
    }

    /// Build `StreamParams` from [`Sirm`].
    pub fn from_sirm(sirm: &Sirm<'_>) -> DeviceResult<Self> {
        let leader_size = sirm.maximum_leader_size()?;
        let trailer_size = sirm.maximum_trailer_size()?;

        let payload_size = sirm.payload_transfer_size()?;
        let payload_count = sirm.payload_transfer_count()?;
        let payload_final1_size = sirm.payload_final_transfer1_size()?;
        let payload_final2_size = sirm.payload_final_transfer2_size()?;

        Ok(Self::new(
            leader_size,
            trailer_size,
            payload_size,
            payload_count,
            payload_final1_size,
            payload_final2_size,
        ))
    }
}
