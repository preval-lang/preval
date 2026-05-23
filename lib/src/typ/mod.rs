use std::collections::HashMap;

mod error;
pub use error::*;
use serde::{Deserialize, Serialize};

use crate::{
    ir::Function, parser::typ::InfoTypeExpr, passes::type_check_expr::Scope,
    value::native::NativeFunction,
};

#[derive(Debug, Clone, Copy, PartialEq, Hash, Serialize, Deserialize)]
pub enum IntegerSize {
    Size,
    Number(usize),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConcreteType {
    Integer { size: IntegerSize, signed: bool },
    Float { size: usize },
    Bool,
    String,
    Struct(HashMap<String, usize>),
    Function(Vec<usize>, usize, Implementation),
    Tuple(Vec<usize>),
    IO,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Type {
    Concrete(ConcreteType),
    Union(usize, usize),
    EarlyReturn,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeExpr {
    Union(Box<InfoTypeExpr>, Box<InfoTypeExpr>),
    Name(String),
    Generics(Box<InfoTypeExpr>, Vec<InfoTypeExpr>),
    Parameter(usize),
    Struct(HashMap<String, InfoTypeExpr>),
    Tuple(Vec<InfoTypeExpr>),

    Integer { size: IntegerSize, signed: bool },
    Float { size: usize },
    Bool,
    String,
    IO,

    Function(Vec<InfoTypeExpr>, Box<InfoTypeExpr>, Implementation),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Implementation {
    Native(NativeFunction),
    Normal(Function),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Instantiator {
    template_names: HashMap<String, InfoTypeExpr>,
    types: Vec<Type>,
}

macro_rules! type_ids {
    ($($name:ident => $expr:expr),* $(,)?) => {
        pub mod type_id {
            type_ids!(@consts [] ; $($name => $expr),*);
        }

        const TYPES: &[TypeExpr] = &[
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
    usize => TypeExpr::Integer { size: IntegerSize::Size, signed: false },
    bool => TypeExpr::Bool,
    empty_tuple => TypeExpr::Tuple(vec![]),
    String => TypeExpr::String,
    IO => TypeExpr::IO,
}

impl Instantiator {
    pub fn new() -> Self {
        let mut template_names = HashMap::new();
        let types = Vec::new();

        for (name, typ) in TYPE_NAMES.iter().zip(TYPES) {
            template_names.insert(
                name.to_string(),
                InfoTypeExpr {
                    expr: typ.clone(),
                    idx: 0,
                },
            );
        }

        Self {
            types,
            template_names,
        }
    }

    pub fn instantiate(&mut self, expr: &InfoTypeExpr, generics: &[usize]) -> usize {
        let type_ = match &expr.expr {
            TypeExpr::Parameter(i) => generics[*i],
            TypeExpr::Name(n) => {
                let template = self
                    .template_names
                    .get(n)
                    .expect(&format!("Type exists {n:?}"))
                    .clone();
                self.instantiate(&template, generics)
            }
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
            TypeExpr::Struct(fields) => {
                let fields = fields
                    .iter()
                    .map(|(k, v)| (k.clone(), self.instantiate(v, generics)))
                    .collect::<HashMap<_, _>>();
                self.add(Type::Concrete(ConcreteType::Struct(fields)))
            }
            TypeExpr::Bool => self.add(Type::Concrete(ConcreteType::Bool)),
            TypeExpr::String => self.add(Type::Concrete(ConcreteType::String)),
            TypeExpr::IO => self.add(Type::Concrete(ConcreteType::IO)),
            TypeExpr::Integer { size, signed } => self.add(Type::Concrete(ConcreteType::Integer {
                size: *size,
                signed: *signed,
            })),
            TypeExpr::Float { size } => {
                self.add(Type::Concrete(ConcreteType::Float { size: *size }))
            }
            TypeExpr::Tuple(elems) => {
                let elems = elems
                    .iter()
                    .map(|e| self.instantiate(e, generics))
                    .collect::<Vec<_>>();
                self.add(Type::Concrete(ConcreteType::Tuple(elems)))
            }
            TypeExpr::Function(args, ret, imp) => {
                let args = args
                    .iter()
                    .map(|e| self.instantiate(e, generics))
                    .collect::<Vec<_>>();
                let ret = self.instantiate(ret, generics);
                self.add(Type::Concrete(ConcreteType::Function(
                    args,
                    ret,
                    imp.clone(),
                )))
            }
        };

        type_
    }

    pub fn add(&mut self, typ: Type) -> usize {
        self.types.push(typ);
        self.types.len() - 1
    }

    pub fn add_template(&mut self, name: String, expr: InfoTypeExpr) {
        self.template_names.insert(name, expr);
    }

    pub fn get_template(&self, name: &str) -> Option<&InfoTypeExpr> {
        self.template_names.get(name)
    }

    pub fn get_type(&self, index: usize) -> Option<&Type> {
        self.types.get(index)
    }

    pub fn compatible(&self, assignee: usize, slot: usize, index: usize) -> Result<bool, ()> {
        let assignee_t = self.get_type(assignee).ok_or(())?;
        if let Type::EarlyReturn = assignee_t {
            return Ok(true);
        }
        let slot_t = self.get_type(slot).ok_or(())?;

        match slot_t {
            Type::Concrete(a) => match assignee_t {
                Type::Concrete(b) => Ok(a == b),
                Type::Union(a, b) => Ok(self.compatible(*a, slot, index + 1)?
                    || self.compatible(*b, slot, index + 1)?),
                Type::EarlyReturn => panic!("Early return can't be assigned"),
            },
            Type::Union(a, b) => Ok(self.compatible(assignee, *a, index + 1)?
                || self.compatible(assignee, *b, index + 1)?),
            Type::EarlyReturn => panic!("Early return can't be a slot"),
        }
    }

    pub fn global_scope<'b>(&self) -> Scope<'b> {
        Scope::new(self.template_names.clone())
    }
}
