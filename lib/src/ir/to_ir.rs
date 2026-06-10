use std::collections::HashMap;

use crate::{
	ir::{
		Block, access::access, error::IRErrorInfo, guard::guard,
		initialize_struct::initialize_struct, is::is,
	},
	parser::expression::{Expr, InfoExpr},
	typ::Program,
};

use crate::ir::{
	block::compile_block, call::call, conditional::conditional, index::index, literal::literal,
	returns::returns, variable::variable, variable_declaration::variable_declaration,
};

pub struct IRContext<'a, 'typ> {
	pub generics: &'a [usize],
	pub ins: &'a mut Program<'typ>,
	pub blocks: &'a mut Vec<Block>,
	pub locals: &'a mut HashMap<String, usize>,
	pub next_var: &'a mut usize,
	pub prefix: &'a [String],
}

impl<'a, 'typ> IRContext<'a, 'typ> {
	pub fn var(&mut self) -> usize {
		*self.next_var += 1;
		*self.next_var
	}
}

pub fn to_ir<'typ>(
	block: &mut usize,
	expr: InfoExpr<'typ>,
	store: Option<usize>,
	tail: bool,
	context: &mut IRContext<'_, 'typ>,
) -> Result<(), IRErrorInfo<'typ>> {
	match expr.expr {
		Expr::Literal(lit) => literal(lit, context.blocks, block, store),
		Expr::Access(left, right) => access(left, right, block, store, context),
		Expr::Let(name, value_expr) => {
			variable_declaration(name, value_expr, block, store, context)
		}
		Expr::Block(statements, returns) => {
			compile_block(statements, returns, block, store, tail, context)
		}
		Expr::InitializeStruct(name, fields) => {
			initialize_struct(name, fields, block, store, context)
		}
		Expr::Return(value_expr) => returns(value_expr, block, context),
		Expr::Call(callee, args) => call(callee, args, block, store, tail, context),
		Expr::Name(name) => variable(name, block, store, context),
		Expr::If { cond, then, els } => {
			conditional(cond, then, els, expr.idx, block, store, tail, context)
		}
		Expr::Guard { dependency, body } => guard(dependency, body, block, store, tail, context),
		Expr::Index(left, right) => index(left, right, block, store, context),
		Expr::Is { name, typ } => is(name, typ, expr.idx, block, store, context),
	}
}
