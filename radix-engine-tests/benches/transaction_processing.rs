use std::collections::BTreeMap;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use radix_engine::types::{NetworkDefinition, NonFungibleIdType, NonFungibleLocalId};
use radix_engine_common::types::Epoch;
use radix_engine_common::ManifestSbor;
use radix_engine_interface::api::node_modules::ModuleConfig;
use radix_engine_interface::blueprints::resource::RoleAssignmentInit;
use radix_engine_interface::blueprints::resource::{NonFungibleResourceRoles, OwnerRole};
use radix_engine_interface::{metadata, metadata_init, ScryptoSbor};
use scrypto::prelude::ComponentAddress;
use scrypto::NonFungibleData;
use transaction::manifest::{compile, decompile, BlobProvider};
use transaction::model::{
    PreparedNotarizedTransactionV1, TransactionHeaderV1, TransactionPayload,
    TransactionPayloadPreparable,
};
use transaction::prelude::*;

fn decompile_notarized_intent_benchmarks(c: &mut Criterion) {
    let compiled_transaction = compiled_notarized_transaction();
    c.bench_function("transaction_processing::prepare", |b| {
        b.iter(|| {
            black_box(
                PreparedNotarizedTransactionV1::prepare_from_payload(&compiled_transaction)
                    .unwrap(),
            )
        })
    });
    c.bench_function("transaction_processing::prepare_and_decompile", |b| {
        b.iter(|| {
            black_box({
                let transaction: PreparedNotarizedTransactionV1 =
                    PreparedNotarizedTransactionV1::prepare_from_payload(&compiled_transaction)
                        .unwrap();
                decompile(
                    &transaction.signed_intent.intent.instructions.inner.0,
                    &NetworkDefinition::simulator(),
                )
                .unwrap()
            })
        })
    });
    c.bench_function(
        "transaction_processing::prepare_and_decompile_and_recompile",
        |b| {
            b.iter(|| {
                black_box({
                    let transaction =
                        PreparedNotarizedTransactionV1::prepare_from_payload(&compiled_transaction)
                            .unwrap();
                    let manifest = decompile(
                        &transaction.signed_intent.intent.instructions.inner.0,
                        &NetworkDefinition::simulator(),
                    )
                    .unwrap();
                    compile(
                        &manifest,
                        &NetworkDefinition::simulator(),
                        BlobProvider::new(),
                    )
                })
            })
        },
    );
}

fn compiled_notarized_transaction() -> Vec<u8> {
    let private_key = Secp256k1PrivateKey::from_u64(1).unwrap();
    let public_key = private_key.public_key();
    let component_address = ComponentAddress::virtual_account_from_public_key(&public_key);

    let manifest = {
        ManifestBuilder::new()
            .lock_fee(component_address, 500)
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                false,
                NonFungibleResourceRoles::default(),
                metadata! {},
                Some(
                    (0u64..10_000u64)
                        .into_iter()
                        .map(|id| (NonFungibleLocalId::integer(id), EmptyStruct {}))
                        .collect::<BTreeMap<NonFungibleLocalId, EmptyStruct>>(),
                ),
            )
            .try_deposit_entire_worktop_or_abort(component_address, None)
            .build()
    };
    let header = TransactionHeaderV1 {
        network_id: 0xf2,
        start_epoch_inclusive: Epoch::of(10),
        end_epoch_exclusive: Epoch::of(13),
        nonce: 0x02,
        notary_public_key: public_key.into(),
        notary_is_signatory: true,
        tip_percentage: 0,
    };
    TransactionBuilder::new()
        .header(header)
        .manifest(manifest)
        .notarize(&private_key)
        .build()
        .to_payload_bytes()
        .unwrap()
}

#[derive(NonFungibleData, ScryptoSbor, ManifestSbor)]
struct EmptyStruct {}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = decompile_notarized_intent_benchmarks
);
criterion_main!(benches);
