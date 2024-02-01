use crate::system::system_db_reader::*;
use crate::system::system_type_checker::BlueprintTypeTarget;
use crate::system::type_info::TypeInfoSubstate;
use crate::track::{
    BatchPartitionStateUpdate, NodeStateUpdates, PartitionStateUpdates, ReadOnly, StateUpdates,
    TrackedNode, TrackedSubstateValue,
};
use crate::types::*;
use radix_engine_interface::blueprints::package::*;
use radix_engine_store_interface::interface::SubstateDatabase;

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub enum SubstateSystemStructure {
    SystemField(SystemFieldStructure),
    SystemSchema,
    // KeyValueStore substates
    KeyValueStoreEntry(KeyValueStoreEntryStructure),
    // Object substates
    ObjectField(FieldStructure),
    ObjectKeyValuePartitionEntry(KeyValuePartitionEntryStructure),
    ObjectIndexPartitionEntry(IndexPartitionEntryStructure),
    ObjectSortedIndexPartitionEntry(SortedIndexPartitionEntryStructure),
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct SystemFieldStructure {
    pub field_kind: SystemFieldKind,
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub enum SystemFieldKind {
    TypeInfo,
    BootLoader,
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct KeyValueStoreEntryStructure {
    pub key_full_type_id: FullyScopedTypeId<NodeId>,
    pub value_full_type_id: FullyScopedTypeId<NodeId>,
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct FieldStructure {
    pub value_schema: ObjectSubstateTypeReference,
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct KeyValuePartitionEntryStructure {
    pub key_schema: ObjectSubstateTypeReference,
    pub value_schema: ObjectSubstateTypeReference,
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct IndexPartitionEntryStructure {
    pub key_schema: ObjectSubstateTypeReference,
    pub value_schema: ObjectSubstateTypeReference,
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct SortedIndexPartitionEntryStructure {
    pub key_schema: ObjectSubstateTypeReference,
    pub value_schema: ObjectSubstateTypeReference,
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub enum ObjectSubstateTypeReference {
    Package(PackageTypeReference),
    ObjectInstance(ObjectInstanceTypeReference),
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct PackageTypeReference {
    pub full_type_id: FullyScopedTypeId<PackageAddress>,
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct ObjectInstanceTypeReference {
    pub instance_type_id: u8,
    pub resolved_full_type_id: FullyScopedTypeId<NodeId>,
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct EventSystemStructure {
    pub package_type_reference: PackageTypeReference,
}

pub type SubstateSystemStructures =
    IndexMap<NodeId, IndexMap<PartitionNumber, IndexMap<SubstateKey, SubstateSystemStructure>>>;

#[derive(Default, Debug, Clone, ScryptoSbor)]
pub struct SystemStructure {
    pub substate_system_structures: SubstateSystemStructures,
    pub event_system_structures: IndexMap<EventTypeIdentifier, EventSystemStructure>,
}

impl SystemStructure {
    pub fn resolve<S: SubstateDatabase>(
        substate_db: &S,
        updates: &IndexMap<NodeId, TrackedNode>,
        application_events: &Vec<(EventTypeIdentifier, Vec<u8>)>,
    ) -> Self {
        let mut substate_schema_mapper =
            SubstateSchemaMapper::new(SystemDatabaseReader::new_with_overlay(substate_db, updates));
        substate_schema_mapper.add_all_written_substate_structures(updates);
        let substate_system_structures = substate_schema_mapper.done();

        let event_system_structures =
            EventSchemaMapper::new(substate_db, &updates, application_events).run();

        SystemStructure {
            substate_system_structures,
            event_system_structures,
        }
    }
}

/// A builder of [`SubstateSystemStructures`].
/// Note that the implementation below assumes that substate owned objects can not be
/// detached. If this changes, we will have to account for objects that are removed
/// from a substate.
pub struct SubstateSchemaMapper<'a, S: SubstateDatabase> {
    /// The source of type information.
    system_reader: SystemDatabaseReader<'a, S>,
    /// The result of the build.
    substate_structures: SubstateSystemStructures,
}

impl<'a, S: SubstateDatabase> SubstateSchemaMapper<'a, S> {
    /// Creates an empty builder.
    pub fn new(system_reader: SystemDatabaseReader<'a, S>) -> Self {
        Self {
            system_reader,
            substate_structures: index_map_new(),
        }
    }

    /// Resolves a [`SubstateSystemStructure`] of the given substate and adds it to the build.
    pub fn add_substate_structure(
        &mut self,
        node_id: &NodeId,
        partition_num: &PartitionNumber,
        key: &SubstateKey,
    ) {
        let partition_descriptors = self
            .system_reader
            .get_partition_descriptors(node_id, partition_num)
            .unwrap();
        let substate_structure =
            self.resolve_substate_structure(node_id, partition_descriptors, key);
        self.substate_structures
            .entry(node_id.clone())
            .or_insert_with(|| index_map_new())
            .entry(partition_num.clone())
            .or_insert_with(|| index_map_new())
            .insert(key.clone(), substate_structure);
    }

    /// A batch `add_substate_structure()` counterpart, tailored for processing all substates
    /// *written* to the track (i.e. skipping reads).
    pub fn add_all_written_substate_structures(&mut self, tracked: &IndexMap<NodeId, TrackedNode>) {
        for (node_id, tracked_node) in tracked {
            for (partition_num, tracked_partition) in &tracked_node.tracked_partitions {
                for (_, tracked_substate) in &tracked_partition.substates {
                    match &tracked_substate.substate_value {
                        TrackedSubstateValue::New(_)
                        | TrackedSubstateValue::ReadExistAndWrite(_, _)
                        | TrackedSubstateValue::ReadNonExistAndWrite(_)
                        | TrackedSubstateValue::WriteOnly(_) => {
                            // The substate has been written - so process this substate structure
                        }
                        TrackedSubstateValue::ReadOnly(ReadOnly::Existent(_))
                        | TrackedSubstateValue::ReadOnly(ReadOnly::NonExistent)
                        | TrackedSubstateValue::Garbage => {
                            // We don't process substates which were only read
                            // NOTE:
                            //   If in future we want to enable this for reads too, it should be possible to
                            //     enable this for TrackedSubstateValue::ReadOnly(ReadOnly::Existent(_))
                            //     but it is not possible for NonExistent reads.
                            //   If a transaction fails, it's possible to get reads of non-existent substates
                            //     where the type info can't be resolved below. For example, if bootstrap fails,
                            //     consensus manager substates are read but the type info is not written.
                            continue;
                        }
                    }

                    self.add_substate_structure(
                        node_id,
                        partition_num,
                        &tracked_substate.substate_key,
                    );
                }
            }
        }
    }

    /// A batch `add_substate_structure()` counterpart, tailored for processing all substates that
    /// were *individually* updated in the given [`StateUpdates`] (i.e. ignoring substates affected
    /// as part of a batch, e.g. during a partition deletion).
    pub fn add_for_all_individually_updated(&mut self, updates: &StateUpdates) {
        for (node_id, node_state_updates) in &updates.by_node {
            match node_state_updates {
                NodeStateUpdates::Delta { by_partition } => {
                    for (partition_num, partition_state_updates) in by_partition {
                        let substate_keys = match partition_state_updates {
                            PartitionStateUpdates::Delta { by_substate } => {
                                by_substate.keys().collect::<Vec<_>>()
                            }
                            PartitionStateUpdates::Batch(BatchPartitionStateUpdate::Reset {
                                new_substate_values,
                            }) => new_substate_values.keys().collect::<Vec<_>>(),
                        };
                        for substate_key in substate_keys {
                            self.add_substate_structure(node_id, partition_num, substate_key);
                        }
                    }
                }
            }
        }
    }

    /// Finalizes the build.
    pub fn done(self) -> SubstateSystemStructures {
        self.substate_structures
    }

    fn resolve_substate_structure(
        &self,
        node_id: &NodeId,
        partition_descriptors: Vec<SystemPartitionDescriptor>,
        key: &SubstateKey,
    ) -> SubstateSystemStructure {
        match &partition_descriptors[0] {
            SystemPartitionDescriptor::BootLoader => {
                SubstateSystemStructure::SystemField(SystemFieldStructure {
                    field_kind: SystemFieldKind::BootLoader,
                })
            }
            SystemPartitionDescriptor::TypeInfo => {
                SubstateSystemStructure::SystemField(SystemFieldStructure {
                    field_kind: SystemFieldKind::TypeInfo,
                })
            }
            SystemPartitionDescriptor::Schema => SubstateSystemStructure::SystemSchema,
            SystemPartitionDescriptor::KeyValueStore => {
                let info = self
                    .system_reader
                    .get_kv_store_type_target(node_id)
                    .unwrap_or_else(|_| panic!("Could not get type info for node {node_id:?}"));

                let key_full_type_id = match info.kv_store_type.key_generic_substitution {
                    GenericSubstitution::Local(type_id) => type_id.under_node(*node_id),
                    GenericSubstitution::Remote(type_id) => self
                        .system_reader
                        .get_blueprint_type_schema(&type_id)
                        .map(|x| x.1.under_node(type_id.package_address.into_node_id()))
                        .unwrap_or_else(|_| panic!("Could not get type info {type_id:?}")),
                };
                let value_full_type_id = match info.kv_store_type.value_generic_substitution {
                    GenericSubstitution::Local(type_id) => type_id.under_node(*node_id),
                    GenericSubstitution::Remote(type_id) => self
                        .system_reader
                        .get_blueprint_type_schema(&type_id)
                        .map(|x| x.1.under_node(type_id.package_address.into_node_id()))
                        .unwrap_or_else(|_| panic!("Could not get type info {type_id:?}")),
                };
                SubstateSystemStructure::KeyValueStoreEntry(KeyValueStoreEntryStructure {
                    key_full_type_id,
                    value_full_type_id,
                })
            }
            SystemPartitionDescriptor::Object(module_id, object_partition_descriptor) => {
                let bp_type_target = self
                    .system_reader
                    .get_blueprint_type_target(node_id, *module_id)
                    .unwrap_or_else(|_| panic!("Could not get type info for node {node_id:?}"));

                self.resolve_object_substate_structure(
                    &bp_type_target,
                    object_partition_descriptor,
                    key,
                )
            }
        }
    }

    fn resolve_object_substate_structure(
        &self,
        bp_type_target: &BlueprintTypeTarget,
        object_partition_descriptor: &ObjectPartitionDescriptor,
        key: &SubstateKey,
    ) -> SubstateSystemStructure {
        match object_partition_descriptor {
            ObjectPartitionDescriptor::Fields => {
                let field_index = match key {
                    SubstateKey::Field(field_index) => field_index,
                    _ => panic!("Invalid field key"),
                };

                let payload_identifier = BlueprintPayloadIdentifier::Field(*field_index);
                let type_reference = self
                    .system_reader
                    .get_blueprint_payload_schema_pointer(&bp_type_target, &payload_identifier)
                    .expect("Could not resolve to type reference");
                return SubstateSystemStructure::ObjectField(FieldStructure {
                    value_schema: type_reference,
                });
            }

            ObjectPartitionDescriptor::KeyValueCollection(collection_index) => {
                let key_identifier =
                    BlueprintPayloadIdentifier::KeyValueEntry(*collection_index, KeyOrValue::Key);
                let value_identifier =
                    BlueprintPayloadIdentifier::KeyValueEntry(*collection_index, KeyOrValue::Value);
                let key_type_reference = self
                    .system_reader
                    .get_blueprint_payload_schema_pointer(&bp_type_target, &key_identifier)
                    .expect("Could not resolve to type reference");
                let value_type_reference = self
                    .system_reader
                    .get_blueprint_payload_schema_pointer(&bp_type_target, &value_identifier)
                    .expect("Could not resolve to type reference");
                SubstateSystemStructure::ObjectKeyValuePartitionEntry(
                    KeyValuePartitionEntryStructure {
                        key_schema: key_type_reference,
                        value_schema: value_type_reference,
                    },
                )
            }

            ObjectPartitionDescriptor::IndexCollection(collection_index) => {
                let key_identifier =
                    BlueprintPayloadIdentifier::IndexEntry(*collection_index, KeyOrValue::Key);
                let value_identifier =
                    BlueprintPayloadIdentifier::IndexEntry(*collection_index, KeyOrValue::Value);
                let key_type_reference = self
                    .system_reader
                    .get_blueprint_payload_schema_pointer(&bp_type_target, &key_identifier)
                    .expect("Could not resolve to type reference");
                let value_type_reference = self
                    .system_reader
                    .get_blueprint_payload_schema_pointer(&bp_type_target, &value_identifier)
                    .expect("Could not resolve to type reference");
                SubstateSystemStructure::ObjectIndexPartitionEntry(IndexPartitionEntryStructure {
                    key_schema: key_type_reference,
                    value_schema: value_type_reference,
                })
            }

            ObjectPartitionDescriptor::SortedIndexCollection(collection_index) => {
                let key_identifier = BlueprintPayloadIdentifier::SortedIndexEntry(
                    *collection_index,
                    KeyOrValue::Key,
                );
                let value_identifier = BlueprintPayloadIdentifier::SortedIndexEntry(
                    *collection_index,
                    KeyOrValue::Value,
                );
                let key_type_reference = self
                    .system_reader
                    .get_blueprint_payload_schema_pointer(&bp_type_target, &key_identifier)
                    .expect("Could not resolve to type reference");
                let value_type_reference = self
                    .system_reader
                    .get_blueprint_payload_schema_pointer(&bp_type_target, &value_identifier)
                    .expect("Could not resolve to type reference");
                SubstateSystemStructure::ObjectSortedIndexPartitionEntry(
                    SortedIndexPartitionEntryStructure {
                        key_schema: key_type_reference,
                        value_schema: value_type_reference,
                    },
                )
            }
        }
    }
}

/// Note that the implementation below assumes that substate owned objects can not be
/// detached. If this changes, we will have to account for objects that are removed
/// from a substate.
pub struct EventSchemaMapper<'a, S: SubstateDatabase> {
    system_reader: SystemDatabaseReader<'a, S>,
    application_events: &'a Vec<(EventTypeIdentifier, Vec<u8>)>,
}

impl<'a, S: SubstateDatabase> EventSchemaMapper<'a, S> {
    pub fn new(
        substate_db: &'a S,
        tracked: &'a IndexMap<NodeId, TrackedNode>,
        application_events: &'a Vec<(EventTypeIdentifier, Vec<u8>)>,
    ) -> Self {
        Self {
            system_reader: SystemDatabaseReader::new_with_overlay(substate_db, tracked),
            application_events,
        }
    }

    pub fn run(&self) -> IndexMap<EventTypeIdentifier, EventSystemStructure> {
        let mut event_system_structures = index_map_new();
        for (event_type_identifier, _) in self.application_events {
            if event_system_structures.contains_key(event_type_identifier) {
                continue;
            }
            let blueprint_id = match &event_type_identifier.0 {
                Emitter::Function(blueprint_id) => blueprint_id.clone(),
                Emitter::Method(node_id, module_id) => {
                    if let ModuleId::Main = module_id {
                        let main_type_info = self.system_reader.get_type_info(node_id).unwrap();
                        match main_type_info {
                            TypeInfoSubstate::Object(info) => info.blueprint_info.blueprint_id,
                            _ => panic!("Unexpected Type Info {:?}", main_type_info),
                        }
                    } else {
                        module_id.static_blueprint().unwrap()
                    }
                }
            };

            let blueprint_definition = self
                .system_reader
                .get_blueprint_definition(&blueprint_id)
                .unwrap();

            let type_pointer = blueprint_definition
                .interface
                .get_event_payload_def(event_type_identifier.1.as_str())
                .unwrap();

            let BlueprintPayloadDef::Static(type_identifier) = type_pointer else {
                panic!("Event identifier type pointer cannot be an instance type pointer");
            };

            let event_system_structure = EventSystemStructure {
                package_type_reference: PackageTypeReference {
                    full_type_id: type_identifier.under_node(blueprint_id.package_address),
                },
            };

            event_system_structures.insert(event_type_identifier.clone(), event_system_structure);
        }

        event_system_structures
    }
}
