use radix_engine::execution::*;
use radix_engine::ledger::*;
use scrypto::abi;
use scrypto::buffer::*;
use scrypto::types::*;
use scrypto::utils::*;

/// Export the ABI of a blueprint.
pub fn export_abi<T: Ledger>(
    ledger: &mut T,
    package: Address,
    blueprint: &str,
    trace: bool,
) -> Result<abi::Blueprint, RuntimeError> {
    let tx_hash = sha256(""); // fixed tx hash for determinism
    let mut mem_ledger = InMemoryLedger::new(); // empty ledger for determinism
    let mut runtime = Runtime::new(tx_hash, &mut mem_ledger);

    // Load package code from file system
    runtime.put_package(
        package,
        ledger
            .get_package(package)
            .ok_or(RuntimeError::PackageNotFound(package))?,
    );

    // Start a process and run abi generator
    let mut process = Process::new(0, trace, &mut runtime);
    let target = process.prepare_call_abi(package, blueprint)?;
    let result = process.run(target);

    // Parse ABI
    scrypto_decode::<abi::Blueprint>(&result?).map_err(|e| RuntimeError::InvalidData(e))
}

/// Export the ABI of the blueprint of a component.
pub fn export_abi_by_component<T: Ledger>(
    ledger: &mut T,
    component: Address,
    trace: bool,
) -> Result<abi::Blueprint, RuntimeError> {
    let com = ledger
        .get_component(component)
        .ok_or(RuntimeError::ComponentNotFound(component))?;

    export_abi(ledger, com.package(), com.blueprint(), trace)
}