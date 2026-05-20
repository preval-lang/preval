mod access;
mod block;
mod call;
mod conditional;
pub mod error;
mod guard;
mod index;
mod initialize_struct;
mod is;
mod literal;
mod printing;
mod returns;
mod to_ir;
mod variable;
mod variable_declaration;

pub use printing::module_to_string;
pub use to_ir::*;

use std::{collections::HashMap, fmt::Debug};

use serde::{Deserialize, Serialize};

use crate::{
    typ::Type,
    value::{PrevalValue, Value, runtime_type::TypeDeserializer},
    vm::{RunResult, evaluate},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Module {
    pub objects: HashMap<String, Value>,
    pub types: HashMap<String, Type>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StructDescriptor {
    pub fields: HashMap<String, usize>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Function {
    pub ir: Vec<Block>,
    pub exported: bool,
}

impl PrevalValue for Function {
    fn get_type(&self) -> TypeDeserializer {
        TypeDeserializer::Function
    }

    fn vcall(&mut self, module: &mut Module, args: Vec<&Option<Value>>) -> RunResult {
        let mut args_map = HashMap::new();
        for (i, arg) in args.iter().enumerate() {
            args_map.insert(i, (**arg).clone());
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
    fn get_type(&self) -> TypeDeserializer {
        TypeDeserializer::Partial
    }

    fn vcall(&mut self, module: &mut Module, args: Vec<&Option<Value>>) -> RunResult {
        let mut args_map: HashMap<usize, Option<Value>> = HashMap::new();
        for (i, arg) in args.iter().enumerate() {
            args_map.insert(i, (**arg).clone());
        }
        evaluate(module, self.blocks.clone(), &mut args_map, self.start_block)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Statement {
    pub store: Option<usize>,
    pub operation: Operation,
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
    GuardPhi {
        block: usize,
        var: usize,
    },
    Index(usize, usize),
    Access(usize, String),
    InitializeStruct(Type, HashMap<String, usize>),
    LoadConstant(String),
    Is {
        value: usize,
        typ: Type,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Callable {
    Var(usize),
    Partial(Partial),
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
    Guard {
        dependency: usize,
        body: usize,
        continuation: usize,
    },
    TailCall {
        function: Callable,
        args: Vec<usize>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub terminal: Terminal,
}
