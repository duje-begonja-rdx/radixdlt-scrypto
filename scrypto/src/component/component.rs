use crate::abi::*;
use crate::engine::scrypto_env::ScryptoEnv;
use crate::runtime::*;
use crate::*;
use radix_engine_interface::api::node_modules::auth::{
    AccessRulesGetLengthInput,
    ACCESS_RULES_GET_LENGTH_IDENT,
};
use radix_engine_interface::api::node_modules::metadata::{
    MetadataSetInput, METADATA_GET_IDENT, METADATA_SET_IDENT,
};
use radix_engine_interface::api::node_modules::royalty::{
    ComponentClaimRoyaltyInput, ComponentSetRoyaltyConfigInput,
    COMPONENT_ROYALTY_CLAIM_ROYALTY_IDENT, COMPONENT_ROYALTY_SET_ROYALTY_CONFIG_IDENT,
};
use radix_engine_interface::api::types::{ComponentId, RENodeId};
use radix_engine_interface::api::{types::*, ClientComponentApi};
use radix_engine_interface::blueprints::resource::{
    require, AccessRule, AccessRules, Bucket, MethodKey,
};
use radix_engine_interface::data::{
    scrypto_decode, scrypto_encode, ScryptoCustomValueKind, ScryptoDecode, ScryptoEncode,
};
use radix_engine_interface::rule;
use sbor::rust::borrow::ToOwned;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;

use super::ComponentAccessRules;

pub trait ComponentState<T: Component + LocalComponent>: ScryptoEncode + ScryptoDecode {
    fn instantiate(self) -> T;
}

// TODO: I've temporarily disabled &mut requirement on the Component trait.
// If not, I will have to overhaul the `ComponentSystem` infra, which will be likely be removed anyway.
//
// Since there is no mutability semantics in the system and kernel, there is no technical benefits
// with &mut, other than to frustrate developers.

pub trait Component {
    fn call<T: ScryptoDecode>(&self, method: &str, args: Vec<u8>) -> T;

    fn set_metadata<K: AsRef<str>, V: AsRef<str>>(&self, name: K, value: V);
    //fn add_access_check(&self, access_rules: AccessRules);
    fn set_royalty_config(&self, royalty_config: RoyaltyConfig);
    fn claim_royalty(&self) -> Bucket;

    fn package_address(&self) -> PackageAddress;
    fn blueprint_name(&self) -> String;
    fn access_rules_chain(&self) -> Vec<ComponentAccessRules>;
    // TODO: fn metadata<K: AsRef<str>>(&self, name: K) -> Option<String>;

    /*
    /// Protects this component with owner badge
    fn with_owner_badge(&self, ) {


        self.add_access_check(access_rules);
    }
     */
}

pub trait LocalComponent: Sized {
    fn globalize(self) -> ComponentAddress;

    fn globalize_with_access_rules(self, access_rules: AccessRules) -> ComponentAddress;

    fn globalize_with_owner_badge(self, owner_badge: NonFungibleGlobalId) -> ComponentAddress {
        let mut access_rules =
            AccessRules::new().default(AccessRule::AllowAll, AccessRule::AllowAll);
        access_rules.set_access_rule_and_mutability(
            MethodKey::new(NodeModuleId::Metadata, METADATA_GET_IDENT.to_string()),
            AccessRule::AllowAll,
            rule!(require(owner_badge.clone())),
        );
        access_rules.set_access_rule_and_mutability(
            MethodKey::new(NodeModuleId::Metadata, METADATA_SET_IDENT.to_string()),
            rule!(require(owner_badge.clone())),
            rule!(require(owner_badge.clone())),
        );
        access_rules.set_access_rule_and_mutability(
            MethodKey::new(
                NodeModuleId::ComponentRoyalty,
                COMPONENT_ROYALTY_SET_ROYALTY_CONFIG_IDENT.to_string(),
            ),
            rule!(require(owner_badge.clone())),
            rule!(require(owner_badge.clone())),
        );
        access_rules.set_access_rule_and_mutability(
            MethodKey::new(
                NodeModuleId::ComponentRoyalty,
                COMPONENT_ROYALTY_CLAIM_ROYALTY_IDENT.to_string(),
            ),
            rule!(require(owner_badge.clone())),
            rule!(require(owner_badge.clone())),
        );

        self.globalize_with_access_rules(access_rules)
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct OwnedComponent(pub ComponentId);

impl Component for OwnedComponent {
    fn call<T: ScryptoDecode>(&self, method: &str, args: Vec<u8>) -> T {
        let output = ScryptoEnv
            .call_method(RENodeId::Component(self.0), method, args)
            .unwrap();
        scrypto_decode(&output).unwrap()
    }

    fn set_metadata<K: AsRef<str>, V: AsRef<str>>(&self, name: K, value: V) {
        ScryptoEnv
            .call_module_method(
                RENodeId::Component(self.0),
                NodeModuleId::Metadata,
                METADATA_SET_IDENT,
                scrypto_encode(&MetadataSetInput {
                    key: name.as_ref().to_owned(),
                    value: value.as_ref().to_owned(),
                })
                .unwrap(),
            )
            .unwrap();
    }

    fn set_royalty_config(&self, royalty_config: RoyaltyConfig) {
        ScryptoEnv
            .call_module_method(
                RENodeId::Component(self.0),
                NodeModuleId::ComponentRoyalty,
                COMPONENT_ROYALTY_SET_ROYALTY_CONFIG_IDENT,
                scrypto_encode(&ComponentSetRoyaltyConfigInput { royalty_config }).unwrap(),
            )
            .unwrap();
    }

    fn claim_royalty(&self) -> Bucket {
        let rtn = ScryptoEnv
            .call_module_method(
                RENodeId::Component(self.0),
                NodeModuleId::ComponentRoyalty,
                COMPONENT_ROYALTY_CLAIM_ROYALTY_IDENT,
                scrypto_encode(&ComponentClaimRoyaltyInput {}).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    fn package_address(&self) -> PackageAddress {
        ScryptoEnv
            .get_component_type_info(RENodeId::Component(self.0))
            .unwrap()
            .0
    }

    fn blueprint_name(&self) -> String {
        ScryptoEnv
            .get_component_type_info(RENodeId::Component(self.0))
            .unwrap()
            .1
    }

    fn access_rules_chain(&self) -> Vec<ComponentAccessRules> {
        let rtn = ScryptoEnv
            .call_module_method(
                RENodeId::Component(self.0),
                NodeModuleId::AccessRules,
                ACCESS_RULES_GET_LENGTH_IDENT,
                scrypto_encode(&AccessRulesGetLengthInput {}).unwrap(),
            )
            .unwrap();

        let length: u32 = scrypto_decode(&rtn).unwrap();
        (0..length)
            .into_iter()
            .map(|id| ComponentAccessRules::new(self.0, id))
            .collect()
    }
}

impl LocalComponent for OwnedComponent {
    fn globalize(self) -> ComponentAddress {
        ScryptoEnv
            .globalize(
                RENodeId::Component(self.0),
                AccessRules::new()
                    .default(AccessRule::AllowAll, AccessRule::DenyAll)
            )
            .unwrap()
    }

    fn globalize_with_access_rules(self, access_rules: AccessRules) -> ComponentAddress {
        ScryptoEnv
            .globalize(RENodeId::Component(self.0), access_rules)
            .unwrap()
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct GlobalComponentRef(pub ComponentAddress);

impl Component for GlobalComponentRef {
    fn call<T: ScryptoDecode>(&self, method: &str, args: Vec<u8>) -> T {
        let output = ScryptoEnv
            .call_method(RENodeId::GlobalComponent(self.0.into()), method, args)
            .unwrap();
        scrypto_decode(&output).unwrap()
    }

    fn set_metadata<K: AsRef<str>, V: AsRef<str>>(&self, name: K, value: V) {
        ScryptoEnv
            .call_module_method(
                RENodeId::GlobalComponent(self.0),
                NodeModuleId::Metadata,
                METADATA_SET_IDENT,
                scrypto_encode(&MetadataSetInput {
                    key: name.as_ref().to_owned(),
                    value: value.as_ref().to_owned(),
                })
                .unwrap(),
            )
            .unwrap();
    }

    fn set_royalty_config(&self, royalty_config: RoyaltyConfig) {
        ScryptoEnv
            .call_module_method(
                RENodeId::GlobalComponent(self.0),
                NodeModuleId::ComponentRoyalty,
                COMPONENT_ROYALTY_SET_ROYALTY_CONFIG_IDENT,
                scrypto_encode(&ComponentSetRoyaltyConfigInput { royalty_config }).unwrap(),
            )
            .unwrap();
    }

    fn claim_royalty(&self) -> Bucket {
        let rtn = ScryptoEnv
            .call_module_method(
                RENodeId::GlobalComponent(self.0),
                NodeModuleId::ComponentRoyalty,
                COMPONENT_ROYALTY_CLAIM_ROYALTY_IDENT,
                scrypto_encode(&ComponentClaimRoyaltyInput {}).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    fn package_address(&self) -> PackageAddress {
        ScryptoEnv
            .get_component_type_info(RENodeId::GlobalComponent(self.0))
            .unwrap()
            .0
    }

    fn blueprint_name(&self) -> String {
        ScryptoEnv
            .get_component_type_info(RENodeId::GlobalComponent(self.0))
            .unwrap()
            .1
    }

    fn access_rules_chain(&self) -> Vec<ComponentAccessRules> {
        let rtn = ScryptoEnv
            .call_module_method(
                RENodeId::GlobalComponent(self.0),
                NodeModuleId::AccessRules,
                ACCESS_RULES_GET_LENGTH_IDENT,
                scrypto_encode(&AccessRulesGetLengthInput {}).unwrap(),
            )
            .unwrap();
        let length: u32 = scrypto_decode(&rtn).unwrap();
        (0..length)
            .into_iter()
            .map(|id| ComponentAccessRules::new(self.0, id))
            .collect()
    }
}

//========
// binary
//========

impl Categorize<ScryptoCustomValueKind> for OwnedComponent {
    #[inline]
    fn value_kind() -> ValueKind<ScryptoCustomValueKind> {
        ValueKind::Custom(ScryptoCustomValueKind::Own)
    }
}

impl<E: Encoder<ScryptoCustomValueKind>> Encode<ScryptoCustomValueKind, E> for OwnedComponent {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        Own::Component(self.0).encode_body(encoder)
    }
}

impl<D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D> for OwnedComponent {
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        let o = Own::decode_body_with_value_kind(decoder, value_kind)?;
        match o {
            Own::Component(component_id) => Ok(Self(component_id)),
            _ => Err(DecodeError::InvalidCustomValue),
        }
    }
}

impl scrypto_abi::LegacyDescribe for OwnedComponent {
    fn describe() -> scrypto_abi::Type {
        Type::Component
    }
}
