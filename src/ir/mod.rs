mod block;
mod call;
mod conditional;
pub mod error;
mod index;
mod literal;
mod printing;
mod returns;
mod to_ir;
mod variable;
mod variable_declaration;

pub use printing::module_to_string;
pub use to_ir::*;

use std::{collections::HashMap, fmt::Debug};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{
    value::{
        PrevalValue, Value,
        typ::{Signature, Type},
    },
    vm::{RunResult, evaluate},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Module {
    pub objects: HashMap<String, Value>,
    pub structs: HashMap<String, StructDescriptor>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StructDescriptor {
    pub fields: IndexMap<String, Type>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct Function {
    pub ir: Vec<Block>,
    pub exported: bool,
    pub signature: Signature,
}

impl PrevalValue for Function {
    fn get_type(&self) -> Type {
        Type::Function(Box::new(self.signature.clone()))
    }

    fn vcall(&self, module: &Module, args: Vec<&Option<Value>>) -> RunResult {
        let mut args_map = HashMap::new();
        for (i, arg) in args.iter().enumerate() {
            args_map.insert(i, arg.clone().clone());
        }
        evaluate(module, self.ir.clone(), &mut args_map, 0)
    }
}

#[derive(PartialEq, Clone, Serialize, Deserialize)]
pub struct Partial {
    pub blocks: Vec<Block>,
    pub start_block: usize,
}
impl PrevalValue for Partial {
    fn get_type(&self) -> Type {
        Type::Partial
    }

    fn vcall(&self, module: &Module, args: Vec<&Option<Value>>) -> RunResult {
        let mut args_map = HashMap::new();
        for (i, arg) in args.iter().enumerate() {
            args_map.insert(i, arg.clone().clone());
        }
        evaluate(module, self.blocks.clone(), &mut args_map, self.start_block)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Statement {
    Operation(Operation, Option<usize>),
    Delete(usize),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Operation {
    Call {
        function: Callable,
        args: Vec<usize>,
    },
    LoadLiteral(Value),
    LoadLocal {
        src: usize,
    },
    Phi {
        block_to_var: HashMap<usize, usize>,
    },
    Index(usize, usize),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Callable {
    Var(usize),
    Partial(Value),
}

#[derive(Debug)]
pub enum Declaration {
    Variable(usize),
    Constant,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Terminal {
    Return(usize),
    Jump(usize),
    CondJump {
        cond: usize,
        then: usize,
        els: usize,
    },
    Branch {
        cond: usize,
        then: RunResult,
        els: RunResult,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub terminal: Terminal,
}
