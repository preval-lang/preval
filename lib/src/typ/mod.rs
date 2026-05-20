use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub mod error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Name {
    pub path: Vec<String>,
    pub generics: Vec<Type>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Type {
    Named(Name),
    Union(Box<Type>, Box<Type>),
    Struct(HashMap<String, Type>, Name),
    Tuple(Vec<Type>),
    Function(Signature),
    EarlyReturn,
    IO,
    Usize,
    Bool,
    String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Signature {
    pub args: Vec<Type>,
    pub returns: Box<Type>,
}

pub mod type_names {
    use super::Type;

    pub fn usize() -> Type {
        Type::Usize
    }

    pub fn string() -> Type {
        Type::String
    }

    pub fn bool() -> Type {
        Type::Bool
    }

    pub fn io() -> Type {
        Type::IO
    }
}
