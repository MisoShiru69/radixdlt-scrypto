use crate::ScryptoSbor;
use radix_engine_common::types::GlobalAddressReservation;
use radix_engine_common::types::NodeId;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct OnVirtualizeInput {
    pub variant_id: u8,
    pub rid: [u8; NodeId::RID_LENGTH],
    pub address_reservation: GlobalAddressReservation,
}

pub type OnVirtualizeOutput = ();

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct OnDropInput {}

pub type OnDropOutput = ();

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct OnMoveInput {
    /// The ID of the node being moved.
    /// Note that there is no guarantee about the validity of the node id.
    pub node_id: NodeId,
    /// True if the node moves from caller to callee, otherwise false.
    pub is_moving_down: bool,
}

pub type OnMoveOutput = ();

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct OnPersistInput {}

pub type OnPersistOutput = ();
