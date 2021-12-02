use sbor::{Decode, Describe, Encode, TypeId};

use crate::buffer::*;
use crate::rust::collections::HashMap;
use crate::rust::vec::Vec;
use crate::types::*;

/// Represents the level of a log message.
#[derive(Debug, Clone, Copy, TypeId, Encode, Decode, Describe, Eq, PartialEq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// Represents the type of a resource.
#[derive(Debug, Clone, Copy, TypeId, Encode, Decode, Describe, Eq, PartialEq)]
pub enum ResourceType {
    /// Represents a fungible resource
    Fungible { granularity: u8 },

    /// Represents a non-fungible resource
    NonFungible,
}

impl ResourceType {
    pub fn granularity(&self) -> u8 {
        match self {
            ResourceType::Fungible { granularity } => *granularity,
            ResourceType::NonFungible => 18,
        }
    }
}

/// Represents som supply of resource.
#[derive(Debug, Clone, TypeId, Encode, Decode, Describe)]
pub enum NewSupply {
    /// A supply of fungible resource represented by amount.
    Fungible { amount: Decimal },

    /// A supply of non-fungible resource represented by a collection of NFTs keyed by ID.
    NonFungible {
        entries: HashMap<u128, (Vec<u8>, Vec<u8>)>,
    },
}

impl NewSupply {
    pub fn fungible<T: Into<Decimal>>(amount: T) -> Self {
        Self::Fungible {
            amount: amount.into(),
        }
    }

    pub fn non_fungible<I: Encode, M: Encode, const N: usize>(entries: [(u128, I, M); N]) -> Self {
        let mut encoded = HashMap::new();
        for (id, i, m) in entries {
            encoded.insert(id, (scrypto_encode(&i), scrypto_encode(&m)));
        }

        Self::NonFungible { entries: encoded }
    }
}