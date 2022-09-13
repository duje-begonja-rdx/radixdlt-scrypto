use transaction::model::ExecutableTransaction;

use crate::engine::AppStateTrack;
use crate::engine::BaseStateTrack;
use crate::engine::StateTrackError;
use crate::engine::*;
use crate::fee::FeeReserve;
use crate::fee::FeeReserveError;
use crate::fee::FeeSummary;
use crate::fee::FeeTable;
use crate::ledger::*;
use crate::model::KeyValueStoreEntrySubstate;
use crate::model::LockableResource;
use crate::model::NonFungibleSubstate;
use crate::model::Resource;
use crate::model::Vault;
use crate::model::VaultSubstate;
use crate::transaction::CommitResult;
use crate::transaction::EntityChanges;
use crate::transaction::RejectResult;
use crate::transaction::TransactionOutcome;
use crate::transaction::TransactionResult;
use crate::types::*;

#[derive(Debug)]
pub enum LockState {
    Read(usize),
    Write,
}

impl LockState {
    pub fn no_lock() -> Self {
        Self::Read(0)
    }
}

#[derive(Debug)]
pub enum SubstateCache {
    Raw(Substate),
    Converted(Vault),
}

impl SubstateCache {
    pub fn raw(&self) -> &Substate {
        match self {
            Self::Raw(substate) => substate,
            Self::Converted(_) => {
                panic!("Attempted to read a raw substate which has been converted into a node")
            }
        }
    }

    pub fn raw_mut(&mut self) -> &mut Substate {
        match self {
            Self::Raw(substate) => substate,
            Self::Converted(_) => {
                panic!("Attempted to read a raw substate which has been converted into a node")
            }
        }
    }

    // Turns substate into a node for dynamic behaviors
    pub fn convert(&mut self) {
        match self {
            SubstateCache::Raw(substate) => {
                let substate: VaultSubstate = substate.clone().into();
                *self = Self::Converted(Vault::new(substate.0));
            }
            SubstateCache::Converted(_) => {}
        }
    }

    pub fn vault(&mut self) -> &Vault {
        match self {
            Self::Raw(_) => panic!("Attempted to read a raw substate as a node"),
            Self::Converted(vault) => vault,
        }
    }

    pub fn vault_mut(&mut self) -> &mut Vault {
        match self {
            Self::Raw(_) => panic!("Attempted to read a raw substate as a node"),
            Self::Converted(vault) => vault,
        }
    }
}

#[derive(Debug)]
pub struct BorrowedSubstate {
    pub substate: SubstateCache,
    pub lock_state: LockState,
}

/// Transaction-wide states and side effects
pub struct Track<'s, R: FeeReserve> {
    application_logs: Vec<(Level, String)>,
    new_substates: Vec<SubstateId>,
    state_track: AppStateTrack<'s>,
    borrowed_substates: HashMap<SubstateId, BorrowedSubstate>,
    pub fee_reserve: R,
    pub fee_table: FeeTable,
}

#[derive(Debug)]
pub enum TrackError {
    NotFound,
    NotAvailable,
    StateTrackError(StateTrackError),
}

pub struct TrackReceipt {
    pub fee_summary: FeeSummary,
    pub application_logs: Vec<(Level, String)>,
    pub result: TransactionResult,
}

impl<'s, R: FeeReserve> Track<'s, R> {
    pub fn new(
        substate_store: &'s dyn ReadableSubstateStore,
        fee_reserve: R,
        fee_table: FeeTable,
    ) -> Self {
        let base_state_track = BaseStateTrack::new(substate_store);
        let state_track = AppStateTrack::new(base_state_track);

        Self {
            application_logs: Vec::new(),
            new_substates: Vec::new(),
            state_track,
            borrowed_substates: HashMap::new(),
            fee_reserve,
            fee_table,
        }
    }

    /// Adds a log message.
    pub fn add_log(&mut self, level: Level, message: String) {
        self.application_logs.push((level, message));
    }

    /// Creates a row with the given key/value
    pub fn create_uuid_substate<V: Into<Substate>>(
        &mut self,
        substate_id: SubstateId,
        value: V,
        is_root: bool,
    ) {
        self.new_substates.push(substate_id.clone());
        self.state_track
            .put_substate(substate_id.clone(), value.into());
        if is_root {
            self.state_track.set_substate_root(substate_id);
        }
    }

    // TODO: Clean this up
    pub fn is_root(&mut self, substate_id: &SubstateId) -> bool {
        self.state_track.is_root(substate_id)
    }

    // TODO: to read/write a value owned by track requires three coordinated steps:
    // 1. Attempt to acquire the lock
    // 2. Apply the operation
    // 3. Release lock
    //
    // A better idea is properly to move the lock-unlock logic into the operation themselves OR to have a
    // representation of locked resource and apply operation on top of it.
    //
    // Also enables us to store state associated with the lock, like the `write_through` flag.

    pub fn acquire_lock(
        &mut self,
        substate_id: SubstateId,
        mutable: bool,
        write_through: bool, // TODO: use a different interface
    ) -> Result<(), TrackError> {
        // Load the substate from state track
        if !self.borrowed_substates.contains_key(&substate_id) {
            if let Some(substate) = self.state_track.get_substate(&substate_id) {
                self.borrowed_substates.insert(
                    substate_id.clone(),
                    BorrowedSubstate {
                        substate: SubstateCache::Raw(substate),
                        lock_state: LockState::no_lock(),
                    },
                );
            } else {
                return Err(TrackError::NotFound);
            }
        }

        let borrowed = self
            .borrowed_substates
            .get_mut(&substate_id)
            .expect("Existence checked upfront");
        match borrowed.lock_state {
            LockState::Read(n) => {
                if mutable {
                    if n != 0 {
                        return Err(TrackError::NotAvailable);
                    }
                    borrowed.lock_state = LockState::Write;
                } else {
                    borrowed.lock_state = LockState::Read(n + 1);
                }
            }
            LockState::Write => {
                return Err(TrackError::NotAvailable);
            }
        }

        Ok(())
    }

    pub fn release_lock(&mut self, substate_id: SubstateId, write_through: bool) {
        let mut borrowed = self
            .borrowed_substates
            .remove(&substate_id)
            .expect("Attempted to release lock on never borrowed substate");

        match &borrowed.lock_state {
            LockState::Read(n) => borrowed.lock_state = LockState::Read(n - 1),
            LockState::Write => borrowed.lock_state = LockState::no_lock(),
        }
    }

    pub fn read_substate(&self, substate_id: SubstateId) -> &SubstateCache {
        &self
            .borrowed_substates
            .get(&substate_id)
            .expect(&format!("Substate {:?} was never locked", substate_id))
            .substate
    }

    pub fn write_substate(&mut self, substate_id: SubstateId) -> &mut SubstateCache {
        &mut self
            .borrowed_substates
            .get_mut(&substate_id)
            .expect(&format!("Substate {:?} was never locked", substate_id))
            .substate
    }

    pub fn take_substate(&mut self, substate_id: SubstateId) -> BorrowedSubstate {
        self.borrowed_substates
            .remove(&substate_id)
            .expect(&format!("Substate {:?} was never locked", substate_id))
    }

    pub fn return_substate(&mut self, substate_id: SubstateId, substate: BorrowedSubstate) {
        self.borrowed_substates.insert(substate_id, substate);
    }

    pub fn read_substate_from_base(
        &self,
        substate_id: SubstateId,
    ) -> Result<Substate, StateTrackError> {
        todo!()
    }

    pub fn write_substate_to_base(
        &self,
        substate_id: SubstateId,
        substate: Substate,
    ) -> Result<Substate, StateTrackError> {
        todo!()
    }

    /// Returns the value of a key value pair
    pub fn read_key_value(&mut self, parent_address: SubstateId, key: Vec<u8>) -> Substate {
        // TODO: consider using a single address as function input
        let substate_id = match parent_address {
            SubstateId::NonFungibleSpace(resource_address) => {
                SubstateId::NonFungible(resource_address, NonFungibleId(key))
            }
            SubstateId::KeyValueStoreSpace(kv_store_id) => {
                SubstateId::KeyValueStoreEntry(kv_store_id, key)
            }
            _ => panic!("Unsupported key value"),
        };

        match parent_address {
            SubstateId::NonFungibleSpace(_) => self
                .state_track
                .get_substate(&substate_id)
                .unwrap_or(Substate::NonFungible(NonFungibleSubstate(None))),
            SubstateId::KeyValueStoreSpace(..) => self
                .state_track
                .get_substate(&substate_id)
                .unwrap_or(Substate::KeyValueStoreEntry(KeyValueStoreEntrySubstate(
                    None,
                ))),
            _ => panic!("Invalid keyed value address {:?}", parent_address),
        }
    }

    /// Sets a key value
    pub fn set_key_value<V: Into<Substate>>(
        &mut self,
        parent_substate_id: SubstateId,
        key: Vec<u8>,
        value: V,
    ) {
        // TODO: consider using a single address as function input
        let substate_id = match parent_substate_id {
            SubstateId::NonFungibleSpace(resource_address) => {
                SubstateId::NonFungible(resource_address, NonFungibleId(key.clone()))
            }
            SubstateId::KeyValueStoreSpace(kv_store_id) => {
                SubstateId::KeyValueStoreEntry(kv_store_id, key.clone())
            }
            _ => panic!("Unsupported key value"),
        };

        self.state_track.put_substate(substate_id, value.into());
    }

    pub fn apply_pre_execution_costs<T: ExecutableTransaction>(
        &mut self,
        transaction: &T,
    ) -> Result<(), FeeReserveError> {
        self.fee_reserve
            .consume(self.fee_table.tx_base_fee(), "base_fee", false)?;

        self.fee_reserve.consume(
            self.fee_table.tx_manifest_decoding_per_byte()
                * transaction.manifest_instructions_size() as u32,
            "decode_manifest",
            false,
        )?;

        self.fee_reserve.consume(
            self.fee_table.tx_manifest_verification_per_byte()
                * transaction.manifest_instructions_size() as u32,
            "verify_manifest",
            false,
        )?;

        self.fee_reserve.consume(
            self.fee_table.tx_signature_verification_per_sig()
                * transaction.signer_public_keys().len() as u32,
            "verify_signatures",
            false,
        )?;

        self.fee_reserve.consume(
            transaction.blobs().iter().map(|b| b.len()).sum::<usize>() as u32
                * self.fee_table.tx_blob_price_per_byte(),
            "blobs",
            true,
        )?;

        Ok(())
    }

    pub fn finalize(
        mut self,
        invoke_result: Result<Vec<Vec<u8>>, RuntimeError>,
        resource_changes: Vec<ResourceChange>, // TODO: wrong abstraction, resource change should be derived from track instead of kernel
    ) -> TrackReceipt {
        let is_success = invoke_result.is_ok();

        // Commit/rollback application state changes
        if is_success {
            self.state_track.commit();
            assert!(self.borrowed_substates.is_empty())
        } else {
            self.state_track.rollback();
            self.borrowed_substates.clear();
            self.new_substates.clear();
        }

        // Close fee reserve
        let fee_summary = self.fee_reserve.finalize();
        let is_rejection = !fee_summary.loan_fully_repaid;

        // Commit fee state changes
        let result = if is_rejection {
            TransactionResult::Reject(RejectResult {
                error: match invoke_result {
                    Ok(..) => RejectionError::SuccessButFeeLoanNotRepaid,
                    Err(error) => RejectionError::ErrorBeforeFeeLoanRepaid(error),
                },
            })
        } else {
            let mut required = fee_summary.burned + fee_summary.tipped;
            let mut collector: LockableResource =
                Resource::new_empty(RADIX_TOKEN, ResourceType::Fungible { divisibility: 18 })
                    .into();
            for (vault_id, mut locked, contingent) in fee_summary.payments.iter().cloned().rev() {
                let amount = if contingent {
                    if is_success {
                        Decimal::min(locked.amount(), required)
                    } else {
                        Decimal::zero()
                    }
                } else {
                    Decimal::min(locked.amount(), required)
                };

                // Deduct fee required
                required = required - amount;

                // Collect fees into collector
                collector
                    .put(
                        locked
                            .take_by_amount(amount)
                            .expect("Failed to extract locked fee"),
                    )
                    .expect("Failed to add fee to fee collector");

                // Refund overpayment
                let substate_id = SubstateId::Vault(vault_id);
                let mut substate = self
                    .state_track
                    .get_substate_from_base(&substate_id)
                    .expect("Failed to fetch a fee-locking vault")
                    .expect("Vault not found");
                substate
                    .vault_mut()
                    .0
                    .put(locked)
                    .expect("Failed to put a fee-locking vault");
                self.state_track.put_substate_to_base(substate_id, substate);
            }

            // TODO: update XRD supply or disable it
            // TODO: pay tips to the lead validator

            let mut new_component_addresses = Vec::new();
            let mut new_resource_addresses = Vec::new();
            let mut new_package_addresses = Vec::new();
            for substate_id in self.new_substates {
                match substate_id {
                    SubstateId::ComponentInfo(component_address) => {
                        new_component_addresses.push(component_address)
                    }
                    SubstateId::ResourceManager(resource_address) => {
                        new_resource_addresses.push(resource_address)
                    }
                    SubstateId::Package(package_address) => {
                        new_package_addresses.push(package_address)
                    }
                    _ => {}
                }
            }

            TransactionResult::Commit(CommitResult {
                outcome: match invoke_result {
                    Ok(output) => TransactionOutcome::Success(output),
                    Err(error) => TransactionOutcome::Failure(error),
                },
                state_updates: self.state_track.into_base().generate_diff(),
                entity_changes: EntityChanges {
                    new_package_addresses,
                    new_component_addresses,
                    new_resource_addresses,
                },
                resource_changes,
            })
        };

        TrackReceipt {
            fee_summary,
            application_logs: self.application_logs,
            result,
        }
    }
}
