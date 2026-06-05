use std::collections::HashMap;

mod error;
pub use error::*;
use serde::{Deserialize, Serialize};

use crate::{ir::Block, parser::typ::InfoTypeExpr, value::native::NativeFunction};

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
    Function(Vec<usize>, usize, Option<Implementation>, Vec<usize>),
    Tuple(Vec<usize>),
    IO,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Type {
    Concrete(ConcreteType),
    Union(usize, usize),
    EarlyReturn,
    Placeholder(usize),
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

    Function(Vec<InfoTypeExpr>, Box<InfoTypeExpr>, Option<Implementation>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Implementation {
    Native(NativeFunction),
    Normal(Vec<Block>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
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

impl Program {
    pub fn new() -> Self {
        let template_names = HashMap::new();
        let types = Vec::new();

        let mut this = Self {
            types,
            template_names,
        };

        for (name, typ) in TYPE_NAMES.iter().zip(TYPES) {
            this.add_template(
                name.to_string(),
                InfoTypeExpr {
                    expr: typ.clone(),
                    idx: 0,
                },
            );
            this.instantiate(
                &InfoTypeExpr {
                    expr: typ.clone(),
                    idx: 0,
                },
                &vec![],
            )
            .unwrap();
        }
        this
    }

    pub fn instantiate(
        &mut self,
        expr: &InfoTypeExpr,
        generics: &[usize],
    ) -> Result<usize, InfoTypeError> {
        let type_ = match &expr.expr {
            TypeExpr::Parameter(i) => generics[*i],
            TypeExpr::Name(n) => {
                let template = match self.template_names.get(n) {
                    Some(temp) => temp.clone(),
                    None => {
                        return Err(InfoTypeError {
                            idx: expr.idx,
                            error: TypeError::UnknownType(n.clone()),
                        });
                    }
                };
                self.instantiate(&template, generics)?
            }
            TypeExpr::Union(a, b) => {
                let a = self.instantiate(a.as_ref(), generics)?;
                let b = self.instantiate(&b, generics)?;
                self.add(Type::Union(a, b))
            }
            TypeExpr::Generics(base, params) => {
                let mut ins_params = Vec::new();

                for param in params {
                    ins_params.push(self.instantiate(param, generics)?);
                }

                self.instantiate(base.as_ref(), &ins_params)?
            }
            TypeExpr::Struct(fields) => {
                let mut ins_fields = HashMap::new();

                for field in fields {
                    ins_fields.insert(field.0.clone(), self.instantiate(field.1, generics)?);
                }
                self.add(Type::Concrete(ConcreteType::Struct(ins_fields)))
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
                let mut ins_elems = Vec::new();

                for elem in elems {
                    ins_elems.push(self.instantiate(elem, generics)?);
                }
                self.add(Type::Concrete(ConcreteType::Tuple(ins_elems)))
            }
            TypeExpr::Function(args, ret, imp) => {
                let mut ins_args = Vec::new();
                for arg in args {
                    ins_args.push(self.instantiate(arg, generics)?);
                }
                let ret = self.instantiate(ret, generics)?;
                self.add(Type::Concrete(ConcreteType::Function(
                    ins_args,
                    ret,
                    imp.clone(),
                    generics.to_vec(),
                )))
            }
        };

        Ok(type_)
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
                Type::Placeholder(_) => Ok(slot_t == assignee_t),
                Type::EarlyReturn => panic!("Early return can't be assigned"),
            },
            Type::Union(a, b) => Ok(self.compatible(assignee, *a, index + 1)?
                || self.compatible(assignee, *b, index + 1)?),
            Type::EarlyReturn => panic!("Early return can't be a slot"),
            Type::Placeholder(_) => Ok(slot_t == assignee_t),
        }
    }
}
