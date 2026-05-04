use std::collections::HashMap;

mod error;
pub use error::*;

use crate::{passes::type_check_expr::Scope, value::runtime_type::RuntimeType};

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum IntegerSize {
    Size,
    Number(usize),
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Signature {
    pub args: Vec<usize>,
    pub returns: usize,
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Concrete(ConcreteType),
    Union(usize, usize),
    EarlyReturn,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum TypeExpr {
    Union(Box<TypeExpr>, Box<TypeExpr>),
    Name(String),
}

pub struct Instantiator {
    visited: HashMap<TypeExpr, usize>,
    names: HashMap<String, usize>,
    types: Vec<Type>,
}

impl Instantiator {
    pub fn new() -> Self {
        let mut names = HashMap::new();
        let mut types = Vec::new();

        let mut names_to_types = HashMap::new();
        names_to_types.insert("IO".to_string(), Type::Concrete(ConcreteType::IO));
        names_to_types.insert("string".to_string(), Type::Concrete(ConcreteType::String));
        names_to_types.insert(
            "usize".to_string(),
            Type::Concrete(ConcreteType::Integer {
                size: IntegerSize::Size,
                signed: false,
            }),
        );

        for name in names_to_types.keys() {
            names.insert(name.clone(), types.len());
            types.push(names_to_types[name].clone());
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

    pub fn instantiate(&mut self, expr: &TypeExpr) -> usize {
        if let Some(&index) = self.visited.get(expr) {
            return index;
        }
        let index = self.types.len();
        self.visited.insert(expr.clone(), index);
        let type_ = match expr {
            TypeExpr::Name(n) => self.types[*self.names.get(n).expect("Type exists")].clone(),
            TypeExpr::Union(a, b) => {
                let a = self.instantiate(a);
                let b = self.instantiate(b);
                Type::Union(a, b)
            }
        };
        self.types.insert(index, type_);
        index
    }

    pub fn concrete(&mut self, concrete: ConcreteType) -> usize {
        println!("TODO: remove concrete function {}:{}", file!(), line!());
        self.add(Type::Concrete(concrete))
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

    pub fn compatible(&self, assignee: usize, slot: usize) -> bool {
        let assignee_t = self.get_type(assignee);
        if let Type::EarlyReturn = assignee_t {
            return true;
        }
        let slot = self.get_type(slot);

        match slot {
            Type::Concrete(a) => match assignee_t {
                Type::Concrete(b) => a == b,
                _ => false,
            },
            Type::Union(a, b) => self.compatible(*a, assignee) || self.compatible(*b, assignee),
            Type::EarlyReturn => panic!("Early return can't be a slot"),
        }
    }

    pub fn global_scope<'b>(&self) -> Scope<'b> {
        Scope::new(self.names.clone())
    }
}
