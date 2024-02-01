use crate::{internal_prelude::*, scenarios::get_builder_for_every_scenario};
use radix_engine::system::system_callback_api::SystemCallbackObject;
use radix_engine::vm::{DefaultNativeVm, NativeVm, NoExtension, Vm};
use radix_engine::{
    system::bootstrap::Bootstrapper,
    vm::{
        wasm::{DefaultWasmEngine, WasmEngine},
        ScryptoVm,
    },
};
use radix_engine_store_interface::interface::*;
use radix_engine_stores::hash_tree_support::HashTreeUpdatingDatabase;
use radix_engine_stores::memory_db::InMemorySubstateDatabase;
use transaction::validation::{NotarizedTransactionValidator, ValidationConfig};

pub struct RunnerContext {
    #[cfg(feature = "std")]
    pub dump_manifest_root: Option<std::path::PathBuf>,
    pub network: NetworkDefinition,
}

#[cfg(feature = "std")]
pub fn run_all_in_memory_and_dump_examples(
    network: NetworkDefinition,
    root_path: std::path::PathBuf,
) -> Result<(), FullScenarioError> {
    let mut event_hasher = HashAccumulator::new();
    let mut substate_db = HashTreeUpdatingDatabase::new(InMemorySubstateDatabase::standard());
    let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
    let native_vm = DefaultNativeVm::new();
    let vm = Vm {
        scrypto_vm: &scrypto_vm,
        native_vm,
    };

    let receipts = Bootstrapper::new(NetworkDefinition::simulator(), &mut substate_db, vm, false)
        .bootstrap_test_default()
        .unwrap();
    let epoch = receipts
        .wrap_up_receipt
        .expect_commit_success()
        .next_epoch()
        .expect("Wrap up ends in next epoch")
        .epoch;

    let mut next_nonce: u32 = 0;
    for scenario_builder in get_builder_for_every_scenario() {
        let mut scenario = scenario_builder(ScenarioCore::new(network.clone(), epoch, next_nonce));
        let context = {
            let sub_folder = root_path.join(scenario.metadata().logical_name);
            // Clear directory before generating anew
            if sub_folder.exists() {
                std::fs::remove_dir_all(&sub_folder).unwrap();
            }

            RunnerContext {
                dump_manifest_root: Some(sub_folder),
                network: network.clone(),
            }
        };
        let end_state = run_scenario_with_default_config(
            &context,
            &mut substate_db,
            &mut scenario,
            &network,
            |hash, receipt| match &receipt.result {
                TransactionResult::Commit(c) => {
                    event_hasher.update_no_chain(hash.as_hash().as_bytes());
                    event_hasher.update_no_chain(scrypto_encode(&c.application_events).unwrap());
                }
                TransactionResult::Reject(_) | TransactionResult::Abort(_) => {}
            },
        )?;
        // TODO(RCnet-V3): Change it so that each scenario starts at a different fixed nonce value, hard-coded for that
        // scenario, to minimize separate scenarios causing non-determinism in others
        next_nonce += 1000;
    }

    assert_eq!(
        substate_db.get_current_root_hash().to_string(),
        "901829c9d41dfbd0d82e08d3b81499f0e591f4b231e90036736c49f47a37ab4e"
    );
    assert_eq!(
        event_hasher.finalize().to_string(),
        "1ae6782ae430f295d509a04882a3d5f1f9bafaedbc46a31144f5699863363fac"
    );

    Ok(())
}

pub fn run_scenario_with_default_config<S, F>(
    context: &RunnerContext,
    substate_db: &mut S,
    scenario: &mut Box<dyn ScenarioInstance>,
    network: &NetworkDefinition,
    mut receipt_handler: F,
) -> Result<EndState, FullScenarioError>
where
    S: SubstateDatabase + CommittableSubstateDatabase,
    F: FnMut(&TransactionIntentHash, &TransactionReceipt),
{
    let costing_parameters = CostingParameters::default();
    let execution_config = ExecutionConfig::for_test_transaction();
    let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
    let native_vm = DefaultNativeVm::new();
    let vm = Vm::new(&scrypto_vm, native_vm);
    let validator = NotarizedTransactionValidator::new(ValidationConfig::default(network.id));

    run_scenario(
        context,
        &validator,
        substate_db,
        vm,
        &costing_parameters,
        &execution_config,
        scenario,
        receipt_handler,
    )
}

pub fn run_scenario<S, V, F>(
    context: &RunnerContext,
    validator: &NotarizedTransactionValidator,
    substate_db: &mut S,
    vm: V,
    costing_parameters: &CostingParameters,
    execution_config: &ExecutionConfig,
    scenario: &mut Box<dyn ScenarioInstance>,
    mut receipt_handler: F,
) -> Result<EndState, FullScenarioError>
where
    S: SubstateDatabase + CommittableSubstateDatabase,
    V: SystemCallbackObject + Clone,
    F: FnMut(&TransactionIntentHash, &TransactionReceipt),
{
    let mut previous = None;
    loop {
        let next = scenario
            .next(previous.as_ref())
            .map_err(|err| err.into_full(&scenario))?;
        match next {
            NextAction::Transaction(next) => {
                let transaction = next
                    .validate(&validator)
                    .map_err(|err| err.into_full(&scenario))?;
                #[cfg(feature = "std")]
                next.dump_manifest(&context.dump_manifest_root, &context.network);
                let receipt = execute_and_commit_transaction(
                    substate_db,
                    vm.clone(),
                    costing_parameters,
                    execution_config,
                    &transaction.get_executable(),
                );
                receipt_handler(transaction.get_executable().intent_hash(), &receipt);
                previous = Some(receipt);
            }
            NextAction::Completed(end_state) => break Ok(end_state),
        }
    }
}

#[cfg(test)]
#[cfg(feature = "std")]
mod test {
    use transaction::manifest::{compile, MockBlobProvider};

    use super::*;

    #[test]
    pub fn update_expected_scenario_output() {
        let network_definition = NetworkDefinition::simulator();
        let scenarios_dir =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("generated-examples");
        run_all_in_memory_and_dump_examples(network_definition.clone(), scenarios_dir.clone())
            .unwrap();

        // Ensure that they can all be compiled back again
        for entry in walkdir::WalkDir::new(&scenarios_dir) {
            let path = entry.unwrap().path().canonicalize().unwrap();
            if path.extension().and_then(|str| str.to_str()) != Some("rtm") {
                continue;
            }

            let manifest_string = std::fs::read_to_string(path).unwrap();
            compile(
                &manifest_string,
                &network_definition,
                MockBlobProvider::new(),
            )
            .unwrap();
        }
    }
}
