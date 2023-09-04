use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::consensus_manager::*;

use crate::prelude::*;

//==================================================================================================
// This file has been autogenerated by the ./update-bindings.sh script and none of the contents here
// are hand-written. If you make any changes to the interface of native blueprints and need to regen
// the bindings then run the update-bindings.sh script at the root of the repo.
//
// Note: there is currently no nice way to format this file since rustfmt doesn't format invocations
// of macros. So, while this is autogenerated, it comes at the price of the stubs readability.
//==================================================================================================

extern_blueprint_internal! { PackageAddress :: new_or_panic ([13u8 , 144u8 , 99u8 , 24u8 , 198u8 , 49u8 , 140u8 , 100u8 , 247u8 , 152u8 , 202u8 , 204u8 , 99u8 , 24u8 , 198u8 , 49u8 , 140u8 , 247u8 , 189u8 , 241u8 , 172u8 , 105u8 , 67u8 , 234u8 , 38u8 , 49u8 , 140u8 , 99u8 , 24u8 , 198u8]) , Faucet , "Faucet" , "OwnedFaucet" , "GlobalFaucet" , FaucetFunctions { fn new (address_reservation : GlobalAddressReservation , bucket : Bucket) -> Global < Faucet > ; } , { fn free (& mut self) -> Bucket ; fn lock_fee (& mut self , amount : Decimal) -> () ; } }
extern_blueprint_internal! { PackageAddress :: new_or_panic ([13u8 , 144u8 , 99u8 , 24u8 , 198u8 , 49u8 , 140u8 , 108u8 , 78u8 , 27u8 , 64u8 , 204u8 , 99u8 , 24u8 , 198u8 , 49u8 , 140u8 , 247u8 , 191u8 , 213u8 , 212u8 , 95u8 , 72u8 , 198u8 , 134u8 , 49u8 , 140u8 , 99u8 , 24u8 , 198u8]) , ConsensusManager , "ConsensusManager" , "OwnedConsensusManager" , "GlobalConsensusManager" , ConsensusManagerFunctions { fn create (validator_owner_token_address : GlobalAddressReservation , component_address : GlobalAddressReservation , initial_epoch : u64 , initial_config : ConsensusManagerConfig , initial_time_ms : i64 , initial_current_leader : Option < u8 >) -> () ; } , { fn get_current_epoch (& self) -> u64 ; fn start (& mut self) -> () ; fn get_current_time (& self , precision : TimePrecision) -> i64 ; fn compare_current_time (& self , instant : i64 , precision : TimePrecision , operator : TimeComparisonOperator) -> bool ; fn next_round (& mut self , round : u64 , proposer_timestamp_ms : i64 , leader_proposal_history : LeaderProposalHistory) -> () ; fn create_validator (& mut self , key : Vec < u8 > , fee_factor : Decimal , xrd_payment : Bucket) -> (ComponentAddress , Bucket , Bucket ,) ; } }

extern_blueprint_internal! { PackageAddress :: new_or_panic ([13u8 , 144u8 , 99u8 , 24u8 , 198u8 , 49u8 , 140u8 , 108u8 , 78u8 , 27u8 , 64u8 , 204u8 , 99u8 , 24u8 , 198u8 , 49u8 , 140u8 , 247u8 , 191u8 , 213u8 , 212u8 , 95u8 , 72u8 , 198u8 , 134u8 , 49u8 , 140u8 , 99u8 , 24u8 , 198u8]) , Validator , "Validator" , "OwnedValidator" , "GlobalValidator" , ValidatorFunctions { } , { fn register (& mut self) -> () ; fn unregister (& mut self) -> () ; fn stake_as_owner (& mut self , stake : Bucket) -> Bucket ; fn stake (& mut self , stake : Bucket) -> Bucket ; fn unstake (& mut self , stake_unit_bucket : Bucket) -> Bucket ; fn claim_xrd (& mut self , bucket : Bucket) -> Bucket ; fn update_key (& mut self , key : Vec < u8 >) -> () ; fn update_fee (& mut self , new_fee_factor : Decimal) -> () ; fn update_accept_delegated_stake (& mut self , accept_delegated_stake : bool) -> () ; fn accepts_delegated_stake (& mut self) -> bool ; fn total_stake_xrd_amount (& self) -> Decimal ; fn total_stake_unit_supply (& self) -> Decimal ; fn get_redemption_value (& self , amount_of_stake_units : Decimal) -> Decimal ; fn signal_protocol_update_readiness (& mut self , vote : String) -> () ; fn get_protocol_update_readiness (& mut self) -> Option < String > ; fn lock_owner_stake_units (& mut self , stake_unit_bucket : Bucket) -> () ; fn start_unlock_owner_stake_units (& mut self , requested_stake_unit_amount : Decimal) -> () ; fn finish_unlock_owner_stake_units (& mut self) -> Bucket ; fn apply_emission (& mut self , xrd_bucket : Bucket , epoch : u64 , proposals_made : u64 , proposals_missed : u64) -> () ; fn apply_reward (& mut self , xrd_bucket : Bucket , epoch : u64) -> () ; } }
extern_blueprint_internal! { PackageAddress :: new_or_panic ([13u8 , 144u8 , 99u8 , 24u8 , 198u8 , 49u8 , 140u8 , 102u8 , 205u8 , 100u8 , 49u8 , 140u8 , 99u8 , 24u8 , 198u8 , 49u8 , 140u8 , 247u8 , 158u8 , 154u8 , 127u8 , 143u8 , 23u8 , 156u8 , 166u8 , 49u8 , 140u8 , 99u8 , 24u8 , 198u8]) , Identity , "Identity" , "OwnedIdentity" , "GlobalIdentity" , IdentityFunctions { fn create_advanced (owner_role : OwnerRole) -> ComponentAddress ; fn create () -> (ComponentAddress , Bucket ,) ; } , { fn securify (& mut self) -> Bucket ; } }
extern_blueprint_internal! { PackageAddress :: new_or_panic ([13u8 , 144u8 , 99u8 , 24u8 , 198u8 , 49u8 , 140u8 , 110u8 , 227u8 , 19u8 , 89u8 , 140u8 , 99u8 , 24u8 , 198u8 , 49u8 , 140u8 , 247u8 , 188u8 , 170u8 , 46u8 , 149u8 , 74u8 , 150u8 , 38u8 , 49u8 , 140u8 , 99u8 , 24u8 , 198u8]) , Account , "Account" , "OwnedAccount" , "GlobalAccount" , AccountFunctions { fn create_advanced (owner_role : OwnerRole , address_reservation : Option < GlobalAddressReservation >) -> ComponentAddress ; fn create () -> (ComponentAddress , Bucket ,) ; } , { fn securify (& mut self) -> Bucket ; fn lock_fee (& mut self , amount : Decimal) -> () ; fn lock_contingent_fee (& mut self , amount : Decimal) -> () ; fn deposit (& mut self , bucket : Bucket) -> () ; fn deposit_batch (& mut self , buckets : Vec < Bucket >) -> () ; fn withdraw (& mut self , resource_address : ResourceAddress , amount : Decimal) -> Bucket ; fn withdraw_non_fungibles (& mut self , resource_address : ResourceAddress , ids : Vec < NonFungibleLocalId >) -> Bucket ; fn burn (& mut self , resource_address : ResourceAddress , amount : Decimal) -> () ; fn burn_non_fungibles (& mut self , resource_address : ResourceAddress , ids : Vec < NonFungibleLocalId >) -> () ; fn lock_fee_and_withdraw (& mut self , amount_to_lock : Decimal , resource_address : ResourceAddress , amount : Decimal) -> Bucket ; fn lock_fee_and_withdraw_non_fungibles (& mut self , amount_to_lock : Decimal , resource_address : ResourceAddress , ids : Vec < NonFungibleLocalId >) -> Bucket ; fn create_proof_of_amount (& self , resource_address : ResourceAddress , amount : Decimal) -> Proof ; fn create_proof_of_non_fungibles (& self , resource_address : ResourceAddress , ids : Vec < NonFungibleLocalId >) -> Proof ; fn set_default_deposit_rule (& self , default : DefaultDepositRule) -> () ; fn set_resource_preference (& self , resource_address : ResourceAddress , resource_preference : ResourcePreference) -> () ; fn remove_resource_preference (& self , resource_address : ResourceAddress) -> () ; fn try_deposit_or_refund (& mut self , bucket : Bucket , authorized_depositor_badge : Option < ResourceOrNonFungible >) -> Option < Bucket > ; fn try_deposit_batch_or_refund (& mut self , buckets : Vec < Bucket > , authorized_depositor_badge : Option < ResourceOrNonFungible >) -> Option < Vec < Bucket >> ; fn try_deposit_or_abort (& mut self , bucket : Bucket , authorized_depositor_badge : Option < ResourceOrNonFungible >) -> () ; fn try_deposit_batch_or_abort (& mut self , buckets : Vec < Bucket > , authorized_depositor_badge : Option < ResourceOrNonFungible >) -> () ; fn add_authorized_depositor (& mut self , badge : ResourceOrNonFungible) -> () ; fn remove_authorized_depositor (& mut self , badge : ResourceOrNonFungible) -> () ; } }
extern_blueprint_internal! { PackageAddress :: new_or_panic ([13u8 , 144u8 , 99u8 , 24u8 , 198u8 , 49u8 , 140u8 , 96u8 , 252u8 , 198u8 , 49u8 , 140u8 , 99u8 , 24u8 , 198u8 , 49u8 , 140u8 , 245u8 , 62u8 , 62u8 , 42u8 , 148u8 , 250u8 , 42u8 , 166u8 , 49u8 , 140u8 , 99u8 , 24u8 , 198u8]) , MultiResourcePool , "MultiResourcePool" , "OwnedMultiResourcePool" , "GlobalMultiResourcePool" , MultiResourcePoolFunctions { fn instantiate (owner_role : OwnerRole , pool_manager_rule : AccessRule , resource_addresses : Vec < ResourceAddress > , address_reservation : Option < GlobalAddressReservation >) -> ComponentAddress ; } , { fn contribute (& mut self , buckets : Vec < Bucket >) -> (Bucket , Vec < Bucket >,) ; fn redeem (& mut self , bucket : Bucket) -> Vec < Bucket > ; fn protected_deposit (& mut self , bucket : Bucket) -> () ; fn protected_withdraw (& mut self , resource_address : ResourceAddress , amount : Decimal , withdraw_strategy : WithdrawStrategy) -> Bucket ; fn get_redemption_value (& self , amount_of_pool_units : Decimal) -> BTreeMap < ResourceAddress , Decimal > ; fn get_vault_amounts (& self) -> BTreeMap < ResourceAddress , Decimal > ; } }

extern_blueprint_internal! { PackageAddress :: new_or_panic ([13u8 , 144u8 , 99u8 , 24u8 , 198u8 , 49u8 , 140u8 , 96u8 , 252u8 , 198u8 , 49u8 , 140u8 , 99u8 , 24u8 , 198u8 , 49u8 , 140u8 , 245u8 , 62u8 , 62u8 , 42u8 , 148u8 , 250u8 , 42u8 , 166u8 , 49u8 , 140u8 , 99u8 , 24u8 , 198u8]) , OneResourcePool , "OneResourcePool" , "OwnedOneResourcePool" , "GlobalOneResourcePool" , OneResourcePoolFunctions { fn instantiate (owner_role : OwnerRole , pool_manager_rule : AccessRule , resource_address : ResourceAddress , address_reservation : Option < GlobalAddressReservation >) -> ComponentAddress ; } , { fn contribute (& mut self , bucket : Bucket) -> Bucket ; fn redeem (& mut self , bucket : Bucket) -> Bucket ; fn protected_deposit (& mut self , bucket : Bucket) -> () ; fn protected_withdraw (& mut self , amount : Decimal , withdraw_strategy : WithdrawStrategy) -> Bucket ; fn get_redemption_value (& self , amount_of_pool_units : Decimal) -> Decimal ; fn get_vault_amount (& self) -> Decimal ; } }

extern_blueprint_internal! { PackageAddress :: new_or_panic ([13u8 , 144u8 , 99u8 , 24u8 , 198u8 , 49u8 , 140u8 , 96u8 , 252u8 , 198u8 , 49u8 , 140u8 , 99u8 , 24u8 , 198u8 , 49u8 , 140u8 , 245u8 , 62u8 , 62u8 , 42u8 , 148u8 , 250u8 , 42u8 , 166u8 , 49u8 , 140u8 , 99u8 , 24u8 , 198u8]) , TwoResourcePool , "TwoResourcePool" , "OwnedTwoResourcePool" , "GlobalTwoResourcePool" , TwoResourcePoolFunctions { fn instantiate (owner_role : OwnerRole , pool_manager_rule : AccessRule , resource_addresses : (ResourceAddress , ResourceAddress ,) , address_reservation : Option < GlobalAddressReservation >) -> ComponentAddress ; } , { fn contribute (& mut self , buckets : (Bucket , Bucket ,)) -> (Bucket , Option < Bucket >,) ; fn redeem (& mut self , bucket : Bucket) -> (Bucket , Bucket ,) ; fn protected_deposit (& mut self , bucket : Bucket) -> () ; fn protected_withdraw (& mut self , resource_address : ResourceAddress , amount : Decimal , withdraw_strategy : WithdrawStrategy) -> Bucket ; fn get_redemption_value (& self , amount_of_pool_units : Decimal) -> BTreeMap < ResourceAddress , Decimal > ; fn get_vault_amounts (& self) -> BTreeMap < ResourceAddress , Decimal > ; } }
extern_blueprint_internal! { PackageAddress :: new_or_panic ([13u8 , 144u8 , 99u8 , 24u8 , 198u8 , 49u8 , 140u8 , 108u8 , 77u8 , 99u8 , 248u8 , 204u8 , 99u8 , 24u8 , 198u8 , 49u8 , 140u8 , 247u8 , 191u8 , 85u8 , 61u8 , 60u8 , 165u8 , 22u8 , 134u8 , 49u8 , 140u8 , 99u8 , 24u8 , 198u8]) , AccessController , "AccessController" , "OwnedAccessController" , "GlobalAccessController" , AccessControllerFunctions { fn create (controlled_asset : Bucket , rule_set : RuleSet , timed_recovery_delay_in_minutes : Option < u32 > , address_reservation : Option < GlobalAddressReservation >) -> ComponentAddress ; } , { fn create_proof (& mut self) -> Proof ; fn initiate_recovery_as_primary (& mut self , rule_set : RuleSet , timed_recovery_delay_in_minutes : Option < u32 >) -> () ; fn initiate_recovery_as_recovery (& mut self , rule_set : RuleSet , timed_recovery_delay_in_minutes : Option < u32 >) -> () ; fn quick_confirm_primary_role_recovery_proposal (& mut self , rule_set : RuleSet , timed_recovery_delay_in_minutes : Option < u32 >) -> () ; fn quick_confirm_recovery_role_recovery_proposal (& mut self , rule_set : RuleSet , timed_recovery_delay_in_minutes : Option < u32 >) -> () ; fn timed_confirm_recovery (& mut self , rule_set : RuleSet , timed_recovery_delay_in_minutes : Option < u32 >) -> () ; fn cancel_primary_role_recovery_proposal (& mut self) -> () ; fn cancel_recovery_role_recovery_proposal (& mut self) -> () ; fn lock_primary_role (& mut self) -> () ; fn unlock_primary_role (& mut self) -> () ; fn stop_timed_recovery (& mut self , rule_set : RuleSet , timed_recovery_delay_in_minutes : Option < u32 >) -> () ; fn initiate_badge_withdraw_attempt_as_primary (& mut self) -> () ; fn initiate_badge_withdraw_attempt_as_recovery (& mut self) -> () ; fn quick_confirm_primary_role_badge_withdraw_attempt (& mut self) -> Bucket ; fn quick_confirm_recovery_role_badge_withdraw_attempt (& mut self) -> Bucket ; fn cancel_primary_role_badge_withdraw_attempt (& mut self) -> () ; fn cancel_recovery_role_badge_withdraw_attempt (& mut self) -> () ; fn mint_recovery_badges (& mut self , non_fungible_local_ids : Vec < NonFungibleLocalId >) -> Bucket ; } }
