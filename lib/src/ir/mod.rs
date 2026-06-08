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
mod returns;
mod to_ir;
mod variable;
mod variable_declaration;

pub use to_ir::*;

use std::{collections::HashMap, fmt::Debug};

use serde::{Deserialize, Serialize};

use crate::{
	typ::{Program, RuntimeTypeExpr},
	value::{PrevalValue, Value, runtime_type::TypeDeserializer},
	vm::{RunResult, evaluate},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct StructDescriptor {
	pub fields: HashMap<String, usize>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Function {
	pub ir: Vec<Block>,
	pub exported: bool,
	pub generics: Vec<usize>,
}

impl PrevalValue for Function {
	fn get_type(&self) -> TypeDeserializer {
		TypeDeserializer::Function
	}

	fn vcall(&mut self, module: &mut Program, args: Vec<&Option<Value>>) -> RunResult {
		let mut args_map = HashMap::new();
		for (i, arg) in args.iter().enumerate() {
			args_map.insert(i, (**arg).clone());
		}
		evaluate(
			module,
			self.ir.clone(),
			&mut args_map,
			0,
			self.generics.clone(),
		)
	}
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Partial {
	pub blocks: Vec<Block>,
	pub start_block: usize,
	pub generics: Vec<usize>,
}
impl PrevalValue for Partial {
	fn get_type(&self) -> TypeDeserializer {
		TypeDeserializer::Partial
	}

	fn vcall(&mut self, module: &mut Program, args: Vec<&Option<Value>>) -> RunResult {
		let mut args_map: HashMap<usize, Option<Value>> = HashMap::new();
		for (i, arg) in args.iter().enumerate() {
			args_map.insert(i, (**arg).clone());
		}
		evaluate(
			module,
			self.blocks.clone(),
			&mut args_map,
			self.start_block,
			self.generics.clone(),
		)
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Statement {
	pub store: Option<usize>,
	pub operation: Operation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
	InitializeStruct(RuntimeTypeExpr, HashMap<String, usize>),
	LoadFunction(RuntimeTypeExpr),
	Is {
		value: usize,
		typ: RuntimeTypeExpr,
	},
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Callable {
	Var(usize),
	Partial(Partial),
}

#[derive(Debug)]
pub enum Declaration {
	Variable(usize),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Block {
	pub statements: Vec<Statement>,
	pub terminal: Terminal,
}
