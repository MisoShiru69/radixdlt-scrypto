use radix_engine::errors::RuntimeError;
use radix_engine_interface::blueprints::account::ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT;
use radix_engine_interface::manifest_args;
use transaction::errors::TransactionValidationError;
use transaction::validation::{NotarizedTransactionValidator, TransactionValidator};

use crate::internal_prelude::*;

use crate::accounts::ed25519_account_1;

pub struct NextTransaction {
    pub logical_name: String,
    pub raw_transaction: RawNotarizedTransaction,
}

impl NextTransaction {
    pub fn of(logical_name: String, transaction: NotarizedTransactionV1) -> Self {
        Self {
            logical_name,
            raw_transaction: transaction.to_raw().expect("Transaction could be encoded"),
        }
    }

    pub fn validate(
        &self,
        validator: &NotarizedTransactionValidator,
    ) -> Result<ValidatedNotarizedTransactionV1, ScenarioError> {
        validator
            .validate_from_raw(&self.raw_transaction)
            .map_err(|err| {
                ScenarioError::TransactionValidationFailed(self.logical_name.clone(), err)
            })
    }
}

pub struct ScenarioContext {
    network: NetworkDefinition,
    epoch: Epoch,
    nonce: u32,
    default_notary: PrivateKey,
    last_transaction_name: Option<String>,
    stage_counter: usize,
    #[cfg(feature = "std")]
    dump_manifest_directory: Option<std::path::PathBuf>,
}

impl ScenarioContext {
    pub fn new(network: NetworkDefinition, epoch: Epoch) -> Self {
        Self {
            network,
            epoch,
            nonce: 0,
            default_notary: ed25519_account_1().key,
            last_transaction_name: None,
            stage_counter: 0,
            #[cfg(feature = "std")]
            dump_manifest_directory: None,
        }
    }

    pub fn next_stage(&mut self) -> usize {
        self.stage_counter += 1;
        self.stage_counter
    }

    #[cfg(feature = "std")]
    pub fn with_manifest_dumping(mut self, directory: std::path::PathBuf) -> Self {
        self.dump_manifest_directory = Some(directory);
        self
    }

    pub fn next_transaction_with_faucet_lock_fee(
        &mut self,
        logical_name: &str,
        create_manifest: impl FnOnce(&mut ManifestBuilder) -> &mut ManifestBuilder,
        signers: Vec<&PrivateKey>,
    ) -> Option<NextTransaction> {
        let mut manifest_builder = ManifestBuilder::new();
        manifest_builder.lock_fee(FAUCET, dec!(100));
        create_manifest(&mut manifest_builder);
        self.next_transaction(logical_name, manifest_builder.build(), signers)
    }

    pub fn next_transaction_free_xrd_from_faucet(
        &mut self,
        to_account: ComponentAddress,
    ) -> Option<NextTransaction> {
        self.next_transaction_with_faucet_lock_fee(
            "faucet-top-up",
            |builder| {
                builder
                    .call_method(FAUCET, "free", manifest_args!())
                    .take_all_from_worktop(XRD, |builder, bucket| {
                        builder.call_method(
                            to_account,
                            ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT,
                            manifest_args!(bucket),
                        )
                    })
            },
            vec![],
        )
    }

    pub fn next_transaction(
        &mut self,
        logical_name: &str,
        manifest: TransactionManifestV1,
        signers: Vec<&PrivateKey>,
    ) -> Option<NextTransaction> {
        let nonce = self.nonce;
        self.nonce += 1;
        #[cfg(feature = "std")]
        self.dump_manifest(logical_name, &manifest);
        let mut builder = TransactionBuilder::new()
            .header(TransactionHeaderV1 {
                network_id: self.network.id,
                start_epoch_inclusive: self.epoch,
                end_epoch_exclusive: self.epoch.next(),
                nonce,
                notary_public_key: self.default_notary.public_key(),
                notary_is_signatory: false,
                tip_percentage: 0,
            })
            .manifest(manifest);
        for signer in signers {
            builder = builder.sign(signer);
        }
        builder = builder.notarize(&self.default_notary);
        self.last_transaction_name = Some(logical_name.to_owned());
        Some(NextTransaction::of(
            logical_name.to_owned(),
            builder.build(),
        ))
    }

    pub fn finish_scenario(&self) -> Option<NextTransaction> {
        None
    }

    #[cfg(feature = "std")]
    fn dump_manifest(&self, logical_name: &str, manifest: &TransactionManifestV1) {
        use transaction::manifest::dumper::dump_manifest_to_file_system;

        let Some(directory_path) = &self.dump_manifest_directory else {
            return;
        };
        let file_name = format!("{}--{}", self.stage_counter, logical_name);
        dump_manifest_to_file_system(manifest, directory_path, Some(&file_name), &self.network)
            .unwrap()
    }

    pub fn network(&self) -> &NetworkDefinition {
        &self.network
    }

    pub fn encoder(&self) -> Bech32Encoder {
        Bech32Encoder::new(&self.network)
    }

    pub fn check_start(&self, previous: &Option<&TransactionReceipt>) -> Result<(), ScenarioError> {
        match previous {
            Some(_) => Err(ScenarioError::PreviousResultProvidedAtStart),
            None => Ok(()),
        }
    }

    pub fn check_commit_success<'a>(
        &self,
        previous: &'a Option<&TransactionReceipt>,
    ) -> Result<&'a CommitResult, ScenarioError> {
        match previous {
            Some(receipt) => match &receipt.result {
                TransactionResult::Commit(c) => match &c.outcome {
                    TransactionOutcome::Success(_) => Ok(c),
                    TransactionOutcome::Failure(err) => Err(ScenarioError::TransactionFailed(
                        self.last_transaction_description(),
                        err.clone(),
                    )),
                },
                TransactionResult::Reject(result) => Err(ScenarioError::TransactionRejected(
                    self.last_transaction_description(),
                    result.clone(),
                )),
                TransactionResult::Abort(result) => Err(ScenarioError::TransactionAborted(
                    self.last_transaction_description(),
                    result.clone(),
                )),
            },
            None => Err(ScenarioError::MissingPreviousResult),
        }
    }

    pub fn last_transaction_description(&self) -> String {
        self.last_transaction_name.clone().unwrap_or("".to_string())
    }
}

#[derive(Debug, Clone)]
pub struct FullScenarioError {
    pub scenario: String,
    pub error: ScenarioError,
}

#[derive(Debug, Clone)]
pub enum ScenarioError {
    PreviousResultProvidedAtStart,
    MissingPreviousResult,
    TransactionFailed(String, RuntimeError),
    TransactionRejected(String, RejectResult),
    TransactionAborted(String, AbortResult),
    TransactionValidationFailed(String, TransactionValidationError),
    Custom(String),
}

impl ScenarioError {
    pub fn into_full(self, scenario: &Box<dyn ScenarioCore>) -> FullScenarioError {
        FullScenarioError {
            scenario: scenario.logical_name().to_owned(),
            error: self,
        }
    }
}

pub trait ScenarioCore {
    /// Gets the logical name of the scenario.
    /// This should be spaceless as it will be used for a file path.
    fn logical_name(&self) -> &'static str;

    /// Consumes the previous receipt, and gets the next transaction in the scenario.
    fn next(
        &mut self,
        context: &mut ScenarioContext,
        previous: Option<&TransactionReceipt>,
    ) -> Result<Option<NextTransaction>, ScenarioError>;
}

pub trait Scenario: Sized + ScenarioCore {
    type Config: Default;

    fn new() -> Self {
        Self::new_with_config(Default::default())
    }

    fn new_with_config(config: Self::Config) -> Self;
}
