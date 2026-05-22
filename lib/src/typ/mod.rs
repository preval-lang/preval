use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub mod error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Name {
    pub path: Vec<String>,
    pub generics: Vec<TypeExpr>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeExpr {
    Named(Name),
    Union(Box<TypeExpr>, Box<TypeExpr>),
    Struct(HashMap<String, TypeExpr>, Vec<String>, Name),
    Tuple(Vec<TypeExpr>),
    Function(Signature),
    EarlyReturn,
    IO,
    Usize,
    Bool,
    String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Signature {
    pub args: Vec<TypeExpr>,
    pub returns: Box<TypeExpr>,
}

pub mod type_names {
    use super::TypeExpr;

    pub fn usize() -> TypeExpr {
        TypeExpr::Usize
    }

    pub fn string() -> TypeExpr {
        TypeExpr::String
    }

    pub fn bool() -> TypeExpr {
        TypeExpr::Bool
    }

    pub fn io() -> TypeExpr {
        TypeExpr::IO
    }
}
