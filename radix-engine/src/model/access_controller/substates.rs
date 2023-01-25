use crate::types::*;
use radix_engine_interface::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerSubstate {
    /// A vault where the asset controlled by the access controller lives.
    pub controlled_asset: VaultId,

    /// The amount of time (in minutes) that it takes for timed recovery to be done. Maximum is
    /// 4,294,967,295 minutes which is 8171.5511700913 years. When this is [`None`], then timed
    /// recovery can not be performed through this access controller.
    pub timed_recovery_delay_in_minutes: Option<u32>,

    /// The states of the Access Controller.
    pub state: (PrimaryRoleState, OperationState),
}

impl AccessControllerSubstate {
    pub fn new(controlled_asset: VaultId, timed_recovery_delay_in_minutes: Option<u32>) -> Self {
        Self {
            controlled_asset,
            timed_recovery_delay_in_minutes,
            state: (PrimaryRoleState::Unlocked, OperationState::Normal),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct RecoveryProposal {
    /// The set of rules being proposed for the different roles.
    pub rule_set: RuleSet,

    /// The proposed delay of timed recoveries.
    pub timed_recovery_delay_in_minutes: Option<u32>,

    /// An [`Instant`] of the time after which timed recovery can be performed. If [`None`], then
    /// timed recovery was not allowed at the time of proposal-initiation.
    pub timed_recovery_allowed_after: Option<Instant>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum PrimaryRoleState {
    Locked,
    Unlocked,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum OperationState {
    Normal,
    Recovery {
        // A mapping of the role that's proposing the recovery to a tuple of the proposed rule set,
        /// proposed `timed_recovery_delay_in_minutes`, and an [`Instant`] of when the timed recovery
        /// delay for this proposal ends. Since [`Proposer`] is used as the key here, we can have a
        /// maximum of two entries in this [`HashMap`] at any given time.
        ongoing_recoveries: HashMap<Proposer, RecoveryProposal>,
    },
}
