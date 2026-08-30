use std::collections::HashMap;

use crate::{
	ir::{
		Block, access::access, guard::guard, initialize_struct::initialize_struct, is::is,
		local::local,
	},
	passes::type_check_expr::{TypedExpr, TypedExprKind},
	typ::Instantiator,
};

use crate::ir::{
	block::compile_block, call::call, conditional::conditional, global::global, index::index,
	literal::literal, returns::returns, variable_declaration::variable_declaration,
};

pub struct IRContext<'a, 'typ> {
	pub generics: &'a [usize],
	pub ins: &'a mut Instantiator<'typ>,
	pub blocks: &'a mut Vec<Block>,
	pub locals: &'a mut HashMap<String, usize>,
	pub next_var: &'a mut usize,
}

impl<'a, 'typ> IRContext<'a, 'typ> {
	pub fn var(&mut self) -> usize {
		*self.next_var += 1;
		*self.next_var
	}
}

pub fn to_ir<'typ>(
	block: &mut usize,
	expr: TypedExpr,
	store: Option<usize>,
	tail: bool,
	context: &mut IRContext<'_, 'typ>,
) {
	match expr.expr {
		TypedExprKind::Literal(lit) => literal(lit, context.blocks, block, store),
		TypedExprKind::Access(left, right) => access(left, right, block, store, context),
		TypedExprKind::Let(name, value_expr) => {
			variable_declaration(name, value_expr, block, store, context)
		}
		TypedExprKind::Block(statements, returns) => {
			compile_block(statements, returns, block, store, tail, context)
		}
		TypedExprKind::InitializeStruct(name, fields) => {
			initialize_struct(name, fields, block, store, context)
		}
		TypedExprKind::Return(value_expr) => returns(value_expr, block, context),
		TypedExprKind::Call(callee, args) => call(callee, args, block, store, tail, context),
		TypedExprKind::Name(name) => global(name, block, store, context),
		TypedExprKind::If { cond, then, els } => {
			conditional(cond, then, els, block, store, tail, context)
		}
		TypedExprKind::Guard { dependency, body } => {
			guard(dependency, body, block, store, tail, context)
		}
		TypedExprKind::Index(left, right) => index(left, right, block, store, context),
		TypedExprKind::Is { name, typ } => is(name, typ, block, store, context),
		TypedExprKind::Local(name) => local(name, block, store, context),
	}
}
