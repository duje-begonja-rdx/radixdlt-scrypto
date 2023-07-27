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
use radix_engine_stores::memory_db::InMemorySubstateDatabase;
use transaction::validation::{NotarizedTransactionValidator, ValidationConfig};

use crate::{internal_prelude::*, scenarios::get_builder_for_every_scenario};

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
    let mut substate_db = InMemorySubstateDatabase::standard();
    let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
    let native_vm = DefaultNativeVm::new();
    let vm = Vm {
        scrypto_vm: &scrypto_vm,
        native_vm,
    };

    let receipts = Bootstrapper::new(&mut substate_db, vm, false)
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
        let end_state =
            run_scenario_with_default_config(&context, &mut substate_db, &mut scenario, &network)?;
        // TODO(RCnet-V3): Change it so that each scenario starts at a different fixed nonce value, hard-coded for that
        // scenario, to minimize separate scenarios causing non-determinism in others
        next_nonce += 1000;
    }
    Ok(())
}

pub fn run_scenario_with_default_config<S>(
    context: &RunnerContext,
    substate_db: &mut S,
    scenario: &mut Box<dyn ScenarioInstance>,
    network: &NetworkDefinition,
) -> Result<EndState, FullScenarioError>
where
    S: SubstateDatabase + CommittableSubstateDatabase,
{
    let fee_reserve_config = FeeReserveConfig::default();
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
        &fee_reserve_config,
        &execution_config,
        scenario,
    )
}

pub fn run_scenario<S, V>(
    context: &RunnerContext,
    validator: &NotarizedTransactionValidator,
    substate_db: &mut S,
    vm: V,
    fee_reserve_config: &FeeReserveConfig,
    execution_config: &ExecutionConfig,
    scenario: &mut Box<dyn ScenarioInstance>,
) -> Result<EndState, FullScenarioError>
where
    S: SubstateDatabase + CommittableSubstateDatabase,
    V: SystemCallbackObject + Clone,
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
                previous = Some(execute_and_commit_transaction(
                    substate_db,
                    vm.clone(),
                    fee_reserve_config,
                    execution_config,
                    &transaction.get_executable(),
                ));
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
    pub fn regenerate_all() {
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