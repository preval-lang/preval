use std::collections::HashMap;

mod error;
pub use error::*;
use serde::{Deserialize, Serialize};

use crate::{parser::typ::InfoTypeExpr, passes::type_check_expr::Scope};

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub enum IntegerSize {
    Size,
    Number(usize),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Signature {
    pub args: Vec<TypeReference>,
    pub returns: TypeReference,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConcreteType {
    Integer { size: IntegerSize, signed: bool },
    Float { size: usize },
    Bool,
    String,
    Struct(HashMap<String, TypeReference>),
    Function(Box<Signature>),
    Tuple(Vec<TypeReference>),
    IO,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeReference {
    Concrete(usize),
    Generic(usize),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Type {
    Concrete(ConcreteType),
    Union(TypeReference, TypeReference),
    EarlyReturn,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum TypeExpr {
    Union(Box<InfoTypeExpr>, Box<InfoTypeExpr>),
    Name(String),
    Generics(Box<InfoTypeExpr>, Vec<InfoTypeExpr>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Instantiator {
    visited: HashMap<TypeExpr, usize>,
    names: HashMap<String, usize>,
    types: Vec<Type>,
}

macro_rules! type_ids {
    ($($name:ident => $expr:expr),* $(,)?) => {
        pub mod type_id {
            type_ids!(@consts [] ; $($name => $expr),*);
        }

        const TYPES: &[ConcreteType] = &[
            $($expr),*
        ];

        const TYPE_NAMES: &[&str] = &[
            $(stringify!($name)),*
        ];
    };

    (@consts [$($seen:ident),*] ; $head:ident => $expr:expr $(, $tail:ident => $tail_expr:expr)*) => {
        #[allow(non_upper_case_globals)]
        #[allow(dead_code)]
        pub const $head: usize = type_ids!(@count $($seen),*);
        type_ids!(@consts [$($seen,)* $head] ; $($tail => $tail_expr),*);
    };

    (@consts [$($seen:ident),*] ;) => {};

    (@count) => { 0 };
    (@count $($idents:ident),*) => {
        <[()]>::len(&[$(type_ids!(@replace $idents)),*])
    };

    (@replace $_t:ident) => { () };
}

type_ids! {
    usize => ConcreteType::Integer { size: IntegerSize::Size, signed: false },
    bool => ConcreteType::Bool,
    empty_tuple => ConcreteType::Tuple(vec![]),
    String => ConcreteType::String,
    IO => ConcreteType::IO,
}

type GenericRestriction = ();

impl Instantiator {
    pub fn new() -> Self {
        let mut names = HashMap::new();
        let mut types = Vec::new();

        let mut names_to_types = Vec::new();
        for (name, typ) in TYPE_NAMES.iter().zip(TYPES) {
            names_to_types.push((name.to_string(), Type::Concrete(typ.clone())));
        }

        for (name, typ) in names_to_types {
            names.insert(name, types.len());
            types.push(typ);
        }

        Self {
            visited: HashMap::new(),
            names,
            types,
        }
    }

    pub fn insert(&mut self, name: &str, typ: Type) -> usize {
        let _ = self.names.insert(name.to_string(), self.types.len());
        self.add(typ)
    }

    pub fn instantiate(
        &mut self,
        expr: &TypeExpr,
        generics: &Vec<(String, GenericRestriction)>,
    ) -> TypeReference {
        if let Some(&index) = self.visited.get(expr) {
            return TypeReference::Concrete(index);
        }
        match expr {
            TypeExpr::Name(n) => {
                let mut type_ref = None;
                for (idx, (name, restriction)) in generics.iter().enumerate() {
                    if n == name {
                        type_ref = Some(TypeReference::Generic(idx));
                        break;
                    }
                }
                if let None = type_ref {
                    type_ref = Some(TypeReference::Concrete(
                        *self.names.get(n).expect(&format!("Type exists {n:?}")),
                    ));
                }

                type_ref.unwrap()
            }
            TypeExpr::Union(a, b) => {
                let a = self.instantiate(a, generics);
                let b = self.instantiate(b, generics);
                let index = self.types.len();
                self.visited.insert(expr.clone(), index);
                self.types.push(Type::Union(a, b));
                TypeReference::Concrete(index)
            }
        }
    }

    pub fn add(&mut self, typ: Type) -> usize {
        self.types.push(typ);
        self.types.len() - 1
    }

    pub fn get_name(&self, name: &str) -> Option<usize> {
        self.names.get(name).copied()
    }

    pub fn get_type(&self, index: usize) -> &Type {
        &self.types[index]
    }

    pub fn compatible(
        &self,
        assignee_ref: TypeReference,
        slot_ref: TypeReference,
        index: usize,
    ) -> bool {
        let slot = if let TypeReference::Generic(slot) = slot_ref {
            slot
        } else {
            return true;
        };

        let assignee = if let TypeReference::Generic(assignee) = assignee_ref {
            assignee
        } else {
            return false;
        };

        let assignee_t = self.get_type(assignee);
        if let Type::EarlyReturn = assignee_t {
            return true;
        }
        let slot_t = self.get_type(slot);

        match slot_t {
            Type::Concrete(a) => match assignee_t {
                Type::Concrete(b) => a == b,
                Type::Union(a, b) => {
                    self.compatible(a.clone(), slot_ref.clone(), index + 1)
                        || self.compatible(b.clone(), slot_ref, index + 1)
                }
                Type::EarlyReturn => panic!("Early return can't be assigned"),
            },
            Type::Union(a, b) => {
                self.compatible(assignee_ref.clone(), a.clone(), index + 1)
                    || self.compatible(assignee_ref, b.clone(), index + 1)
            }
            Type::EarlyReturn => panic!("Early return can't be a slot"),
        }
    }

    pub fn global_scope<'b>(&self) -> Scope<'b> {
        Scope::new(
            self.names
                .iter()
                .map(|entry| (entry.0.clone(), TypeReference::Concrete(*entry.1)))
                .collect(),
        )
    }
}
