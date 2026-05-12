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

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct Signature {
    pub args: Vec<usize>,
    pub returns: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConcreteType {
    Integer { size: IntegerSize, signed: bool },
    Float { size: usize },
    Bool,
    String,
    Struct(HashMap<String, usize>),
    Function(Box<Signature>),
    Tuple(Vec<usize>),
    IO,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Type {
    Concrete(ConcreteType),
    Union(usize, usize),
    EarlyReturn,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum TypeExpr {
    Union(Box<InfoTypeExpr>, Box<InfoTypeExpr>),
    Name(String),
    Generics(Box<InfoTypeExpr>, Vec<InfoTypeExpr>),
    Parameter(usize),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Instantiator {
    visited: HashMap<TypeExpr, usize>,
    names: HashMap<String, usize>,
    template_names: HashMap<String, TypeExpr>,
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
            template_names: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: &str, typ: Type) -> usize {
        let _ = self.names.insert(name.to_string(), self.types.len());
        self.add(typ)
    }

    pub fn instantiate(&mut self, expr: &InfoTypeExpr, generics: &[usize]) -> usize {
        if let Some(&index) = self.visited.get(&expr.expr) {
            return index;
        }
        let type_ = match &expr.expr {
            TypeExpr::Parameter(i) => generics[*i],
            TypeExpr::Name(n) => *self.names.get(n).expect(&format!("Type exists {n:?}")),
            TypeExpr::Union(a, b) => {
                let a = self.instantiate(a.as_ref(), generics);
                let b = self.instantiate(&b, generics);
                self.add(Type::Union(a, b))
            }
            TypeExpr::Generics(base, params) => {
                let params = params
                    .iter()
                    .map(|p| self.instantiate(p, generics))
                    .collect::<Vec<_>>();

                self.instantiate(base.as_ref(), &params)
            }
        };

        self.visited.insert(expr.expr.clone(), type_);

        type_
    }

    pub fn add(&mut self, typ: Type) -> usize {
        self.types.push(typ);
        self.types.len() - 1
    }

    pub fn add_template(&mut self, name: String, expr: TypeExpr) {
        self.template_names.insert(name, expr);
    }

    pub fn get_name(&self, name: &str) -> Option<usize> {
        self.names.get(name).copied()
    }

    pub fn get_type(&self, index: usize) -> &Type {
        &self.types[index]
    }

    pub fn compatible(&self, assignee: usize, slot: usize, index: usize) -> bool {
        let assignee_t = self.get_type(assignee);
        if let Type::EarlyReturn = assignee_t {
            return true;
        }
        let slot_t = self.get_type(slot);

        match slot_t {
            Type::Concrete(a) => match assignee_t {
                Type::Concrete(b) => a == b,
                Type::Union(a, b) => {
                    self.compatible(*a, slot, index + 1) || self.compatible(*b, slot, index + 1)
                }
                Type::EarlyReturn => panic!("Early return can't be assigned"),
            },
            Type::Union(a, b) => {
                self.compatible(assignee, *a, index + 1) || self.compatible(assignee, *b, index + 1)
            }
            Type::EarlyReturn => panic!("Early return can't be a slot"),
        }
    }

    pub fn global_scope<'b>(&self) -> Scope<'b> {
        Scope::new(self.names.clone())
    }
}
