use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use radix_engine::types::*;
use radix_engine_common::crypto::{recover_secp256k1, verify_secp256k1};
use transaction::prelude::*;
use transaction::validation::*;

fn bench_secp256k1_validation(c: &mut Criterion) {
    let message_hash = hash("This is a long message".repeat(100));
    let signer = Secp256k1PrivateKey::from_u64(123123123123).unwrap();
    let signature = signer.sign(&message_hash);

    c.bench_function("transaction_validation::verify_ecdsa", |b| {
        b.iter(|| {
            let public_key = recover_secp256k1(&message_hash, &signature).unwrap();
            verify_secp256k1(&message_hash, &public_key, &signature);
        })
    });
}

fn bench_ed25519_validation(c: &mut Criterion) {
    let message_hash = hash("This is a long message".repeat(100));
    let signer = Ed25519PrivateKey::from_u64(123123123123).unwrap();
    let public_key = signer.public_key();
    let signature = signer.sign(&message_hash);

    c.bench_function("transaction_validation::verify_ed25519", |b| {
        b.iter(|| {
            verify_ed25519(&message_hash, &public_key, &signature);
        })
    });
}

fn bench_bls_validation_long(c: &mut Criterion) {
    let message = vec![0u8; 2048];
    println!("message len = {}", message.len());
    let signer = Bls12381G1PrivateKey::from_u64(123123123123).unwrap();
    let public_key = signer.public_key();
    let signature = signer.sign_v1(&message);

    c.bench_function("transaction_validation::verify_bls_2KB", |b| {
        b.iter(|| {
            verify_bls12381_v1(&message, &public_key, &signature);
        })
    });
}

fn bench_bls_validation_short(c: &mut Criterion) {
    let message = vec![0u8; 32];
    let signer = Bls12381G1PrivateKey::from_u64(123123123123).unwrap();
    let public_key = signer.public_key();
    let signature = signer.sign_v1(&message);

    c.bench_function("transaction_validation::verify_bls_32B", |b| {
        b.iter(|| {
            verify_bls12381_v1(&message, &public_key, &signature);
        })
    });
}

fn bench_transaction_validation(c: &mut Criterion) {
    let address_bech32_decoder: AddressBech32Decoder =
        AddressBech32Decoder::new(&NetworkDefinition::simulator());

    let account1 = ComponentAddress::try_from_bech32(
        &address_bech32_decoder,
        "account_sim1cyvgx33089ukm2pl97pv4max0x40ruvfy4lt60yvya744cve475w0q",
    )
    .unwrap();
    let account2 = ComponentAddress::try_from_bech32(
        &address_bech32_decoder,
        "account_sim1cyzfj6p254jy6lhr237s7pcp8qqz6c8ahq9mn6nkdjxxxat5syrgz9",
    )
    .unwrap();
    let signer = Secp256k1PrivateKey::from_u64(1).unwrap();

    let transaction = TransactionBuilder::new()
        .header(TransactionHeaderV1 {
            network_id: NetworkDefinition::simulator().id,
            start_epoch_inclusive: Epoch::zero(),
            end_epoch_exclusive: Epoch::of(100),
            nonce: 1,
            notary_public_key: signer.public_key().into(),
            notary_is_signatory: true,
            tip_percentage: 5,
        })
        .manifest(
            ManifestBuilder::new()
                .withdraw_from_account(account1, XRD, 1)
                .try_deposit_entire_worktop_or_abort(account2, None)
                .build(),
        )
        .notarize(&signer)
        .build();
    let transaction_bytes = transaction.to_payload_bytes().unwrap();
    println!("Transaction size: {} bytes", transaction_bytes.len());

    let validator = NotarizedTransactionValidator::new(ValidationConfig::simulator());

    c.bench_function("transaction_validation::validate_manifest", |b| {
        b.iter(|| {
            black_box(
                validator
                    .validate_from_payload_bytes(&transaction_bytes)
                    .unwrap(),
            )
        })
    });
}

criterion_group!(
    validation,
    bench_secp256k1_validation,
    bench_ed25519_validation,
    bench_bls_validation_short,
    bench_bls_validation_long,
    bench_transaction_validation,
);
criterion_main!(validation);
