extern crate alloc;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Component {
    pub name: String,
    pub methods: Vec<Method>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Method {
    pub name: String,
    pub kind: MethodKind,
    pub mutability: Mutability,
    pub inputs: Vec<Type>,
    pub output: Type,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MethodKind {
    Functional,
    Stateful,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Mutability {
    Immutable,
    Mutable,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Type {
    /* unit */
    Void,

    /* boolean */
    Bool,

    /* integers */
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,

    /* String, &str */
    String,

    /* Option<T> */
    Option {
        value: Box<Type>,
    },

    /* [T] */
    Array {
        base: Box<Type>,
    },

    /* (A, B, C) */
    Tuple {
        elements: Vec<Type>,
    },

    /* struct */
    Struct {
        name: String,
        attributes: BTreeMap<String, Type>,
    },
}