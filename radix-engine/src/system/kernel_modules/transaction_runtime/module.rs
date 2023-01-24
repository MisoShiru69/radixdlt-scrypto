use crate::{
    errors::RuntimeError,
    kernel::kernel_api::SubstateApi,
    kernel::{CallFrameUpdate, ResolvedActor},
};
use radix_engine_interface::api::types::RENodeId;

pub struct TransactionHashModule;

impl TransactionHashModule {
    pub fn on_call_frame_enter<Y: SubstateApi>(
        call_frame_update: &mut CallFrameUpdate,
        _actor: &ResolvedActor,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let refed = api.get_visible_nodes()?;
        let maybe_hash_id = refed
            .into_iter()
            .find(|e| matches!(e, RENodeId::TransactionRuntime(..)));
        if let Some(transaction_hash_id) = maybe_hash_id {
            call_frame_update
                .node_refs_to_copy
                .insert(transaction_hash_id);
        }

        Ok(())
    }
}
