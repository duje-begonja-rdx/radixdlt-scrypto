use scrypto::prelude::*;

#[derive(NonFungibleData, ScryptoSbor)]
struct DummyNFData {
    name: String,
}

#[blueprint]
mod pc {
    struct ProofCreation {
        vault: Vault,
    }

    impl ProofCreation {
        //==================
        // Bucket
        //==================

        pub fn create_proof_from_fungible_bucket() {
            let bucket = Self::create_fungible_bucket();
            let proof = bucket.create_proof().no_check();
            assert_eq!(proof.amount(), dec!(1));
            proof.drop();
            bucket.burn();
        }
        pub fn create_proof_from_fungible_bucket_of_amount() {
            let bucket = Self::create_fungible_bucket();
            let proof = bucket.create_proof_of_amount(2).no_check();
            assert_eq!(proof.amount(), dec!(2));
            proof.drop();
            bucket.burn();
        }
        pub fn create_proof_from_fungible_bucket_of_non_fungibles() {
            let bucket = Self::create_fungible_bucket();
            let proof = bucket
                .create_proof_of_non_fungibles(btreeset!(
                    NonFungibleLocalId::integer(1),
                    NonFungibleLocalId::integer(2)
                ))
                .no_check();
            assert_eq!(proof.amount(), dec!(2));
            proof.drop();
            bucket.burn();
        }
        pub fn create_proof_from_fungible_bucket_of_all() {
            let bucket = Self::create_fungible_bucket();
            let proof = bucket.create_proof_of_all().no_check();
            assert_eq!(proof.amount(), dec!(100));
            proof.drop();
            bucket.burn();
        }

        pub fn create_proof_from_non_fungible_bucket() {
            let bucket = Self::create_non_fungible_bucket();
            let proof = bucket.create_proof().no_check();
            assert_eq!(proof.amount(), dec!(1));
            proof.drop();
            bucket.burn();
        }
        pub fn create_proof_from_non_fungible_bucket_of_amount() {
            let bucket = Self::create_non_fungible_bucket();
            let proof = bucket.create_proof_of_amount(2).no_check();
            assert_eq!(proof.amount(), dec!(2));
            proof.drop();
            bucket.burn();
        }
        pub fn create_proof_from_non_fungible_bucket_of_non_fungibles() {
            let bucket = Self::create_non_fungible_bucket();
            let proof = bucket
                .create_proof_of_non_fungibles(btreeset!(
                    NonFungibleLocalId::integer(1),
                    NonFungibleLocalId::integer(2)
                ))
                .no_check();
            assert_eq!(proof.amount(), dec!(2));
            proof.drop();
            bucket.burn();
        }
        pub fn create_proof_from_non_fungible_bucket_of_all() {
            let bucket = Self::create_non_fungible_bucket();
            let proof = bucket.create_proof_of_all().no_check();
            assert_eq!(proof.amount(), dec!(3));
            proof.drop();
            bucket.burn();
        }

        //==================
        // Vault
        //==================

        pub fn create_proof_from_fungible_vault() {
            let vault = Self::create_fungible_vault();
            let proof = vault.create_proof().no_check();
            assert_eq!(proof.amount(), dec!(1));
            proof.drop();
            ProofCreation { vault }.instantiate().globalize();
        }
        pub fn create_proof_from_fungible_vault_of_amount() {
            let vault = Self::create_fungible_vault();
            let proof = vault.create_proof_of_amount(2).no_check();
            assert_eq!(proof.amount(), dec!(2));
            proof.drop();
            ProofCreation { vault }.instantiate().globalize();
        }
        pub fn create_proof_from_fungible_vault_of_non_fungibles() {
            let vault = Self::create_fungible_vault();
            let proof = vault
                .create_proof_of_non_fungibles(btreeset!(
                    NonFungibleLocalId::integer(1),
                    NonFungibleLocalId::integer(2)
                ))
                .no_check();
            assert_eq!(proof.amount(), dec!(2));
            proof.drop();
            ProofCreation { vault }.instantiate().globalize();
        }

        pub fn create_proof_from_non_fungible_vault() {
            let vault = Self::create_non_fungible_vault();
            let proof = vault.create_proof().no_check();
            assert_eq!(proof.amount(), dec!(1));
            proof.drop();
            ProofCreation { vault }.instantiate().globalize();
        }
        pub fn create_proof_from_non_fungible_vault_of_amount() {
            let vault = Self::create_non_fungible_vault();
            let proof = vault.create_proof_of_amount(2).no_check();
            assert_eq!(proof.amount(), dec!(2));
            proof.drop();
            ProofCreation { vault }.instantiate().globalize();
        }
        pub fn create_proof_from_non_fungible_vault_of_non_fungibles() {
            let vault = Self::create_non_fungible_vault();
            let proof = vault
                .create_proof_of_non_fungibles(btreeset!(
                    NonFungibleLocalId::integer(1),
                    NonFungibleLocalId::integer(2)
                ))
                .no_check();
            assert_eq!(proof.amount(), dec!(2));
            proof.drop();
            ProofCreation { vault }.instantiate().globalize();
        }

        //==================
        // Auth Zone
        //==================

        pub fn create_proof_from_fungible_auth_zone() {
            let bucket = Self::prepare_fungible_proof();
            let proof = LocalAuthZone::create_proof(bucket.resource_address()).no_check();
            assert_eq!(proof.amount(), dec!(1));
            proof.drop();
            LocalAuthZone::clear();
            bucket.burn();
        }
        pub fn create_proof_from_fungible_auth_zone_of_amount() {
            let bucket = Self::prepare_fungible_proof();
            let proof =
                LocalAuthZone::create_proof_of_amount(2, bucket.resource_address()).no_check();
            assert_eq!(proof.amount(), dec!(2));
            proof.drop();
            LocalAuthZone::clear();
            bucket.burn();
        }
        pub fn create_proof_from_fungible_auth_zone_of_non_fungibles() {
            let bucket = Self::prepare_fungible_proof();
            let proof = LocalAuthZone::create_proof_of_non_fungibles(
                btreeset!(
                    NonFungibleLocalId::integer(1),
                    NonFungibleLocalId::integer(2)
                ),
                bucket.resource_address(),
            )
            .no_check();
            assert_eq!(proof.amount(), dec!(2));
            proof.drop();
            LocalAuthZone::clear();
            bucket.burn();
        }
        pub fn create_proof_from_fungible_auth_zone_of_all() {
            let bucket = Self::prepare_fungible_proof();
            let proof = LocalAuthZone::create_proof_of_all(bucket.resource_address()).no_check();
            assert_eq!(proof.amount(), dec!(100));
            proof.drop();
            LocalAuthZone::clear();
            bucket.burn();
        }

        pub fn create_proof_from_non_fungible_auth_zone() {
            let bucket = Self::prepare_non_fungible_proof();
            let proof = LocalAuthZone::create_proof(bucket.resource_address()).no_check();
            assert_eq!(proof.amount(), dec!(1));
            proof.drop();
            LocalAuthZone::clear();
            bucket.burn();
        }
        pub fn create_proof_from_non_fungible_auth_zone_of_amount() {
            let bucket = Self::prepare_non_fungible_proof();
            let proof =
                LocalAuthZone::create_proof_of_amount(2, bucket.resource_address()).no_check();
            assert_eq!(proof.amount(), dec!(2));
            proof.drop();
            LocalAuthZone::clear();
            bucket.burn();
        }
        pub fn create_proof_from_non_fungible_auth_zone_of_non_fungibles() {
            let bucket = Self::prepare_non_fungible_proof();
            let proof = LocalAuthZone::create_proof_of_non_fungibles(
                btreeset!(
                    NonFungibleLocalId::integer(1),
                    NonFungibleLocalId::integer(2)
                ),
                bucket.resource_address(),
            )
            .no_check();
            assert_eq!(proof.amount(), dec!(2));
            proof.drop();
            LocalAuthZone::clear();
            bucket.burn();
        }
        pub fn create_proof_from_non_fungible_auth_zone_of_all() {
            let bucket = Self::prepare_non_fungible_proof();
            let proof = LocalAuthZone::create_proof_of_all(bucket.resource_address()).no_check();
            assert_eq!(proof.amount(), dec!(3));
            proof.drop();
            LocalAuthZone::clear();
            bucket.burn();
        }

        //==================
        // helper functions
        //==================

        pub fn create_fungible_bucket() -> Bucket {
            ResourceBuilder::new_fungible()
                .burnable(AccessRule::AllowAll, AccessRule::DenyAll)
                .mint_initial_supply(100)
        }

        pub fn create_non_fungible_bucket() -> Bucket {
            ResourceBuilder::new_integer_non_fungible()
                .burnable(AccessRule::AllowAll, AccessRule::DenyAll)
                .mint_initial_supply([
                    (
                        1u64.into(),
                        DummyNFData {
                            name: "NF One".to_owned(),
                        },
                    ),
                    (
                        2u64.into(),
                        DummyNFData {
                            name: "NF Two".to_owned(),
                        },
                    ),
                    (
                        3u64.into(),
                        DummyNFData {
                            name: "NF three".to_owned(),
                        },
                    ),
                ])
        }

        pub fn create_fungible_vault() -> Vault {
            Vault::with_bucket(Self::create_fungible_bucket())
        }

        pub fn create_non_fungible_vault() -> Vault {
            Vault::with_bucket(Self::create_non_fungible_bucket())
        }

        pub fn prepare_fungible_proof() -> Bucket {
            let bucket = Self::create_fungible_bucket();
            LocalAuthZone::push(bucket.create_proof_of_all());
            bucket
        }

        pub fn prepare_non_fungible_proof() -> Bucket {
            let bucket = Self::create_non_fungible_bucket();
            LocalAuthZone::push(bucket.create_proof_of_all());
            bucket
        }
    }
}
