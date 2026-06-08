use std::collections::HashMap;

use crate::{
	ir::{
		Block, Declaration, access::access, error::IRErrorInfo, guard::guard,
		initialize_struct::initialize_struct, is::is,
	},
	parser::expression::{Expr, InfoExpr},
};

use crate::ir::{
	block::compile_block, call::call, conditional::conditional, index::index, literal::literal,
	returns::returns, variable::variable, variable_declaration::variable_declaration,
};

pub fn to_ir<'a>(
	ir: &mut Vec<Block>,
	block: &mut usize,
	expr: InfoExpr<'a>,
	store: Option<usize>,
	locals: &mut HashMap<String, Declaration>,
	next_var: &mut usize,
	tail: bool,
) -> Result<(), IRErrorInfo<'a>> {
	match expr.expr {
		Expr::Literal(lit) => literal(lit, ir, block, store),
		Expr::Access(left, right) => access(left, right, ir, block, store, locals, next_var),
		Expr::Let(name, value_expr) => {
			variable_declaration(name, value_expr, ir, block, store, locals, next_var)
		}
		Expr::Block(statements, returns) => compile_block(
			statements, returns, ir, block, store, locals, next_var, tail,
		),
		Expr::InitializeStruct(name, fields) => {
			initialize_struct(name, fields, ir, block, store, locals, next_var)
		}
		Expr::Return(value_expr) => returns(value_expr, ir, block, locals, next_var),
		Expr::Call(callee, args) => call(callee, args, ir, block, store, locals, next_var, tail),
		Expr::Name(name) => variable(name, ir, block, store, locals),
		Expr::If { cond, then, els } => conditional(
			cond, then, els, expr.idx, ir, block, store, locals, next_var, tail,
		),
		Expr::Guard { dependency, body } => {
			guard(dependency, body, ir, block, store, locals, next_var, tail)
		}
		Expr::Index(left, right) => index(left, right, ir, block, store, locals, next_var),
		Expr::Is { name, typ } => is(name, typ, expr.idx, ir, block, store, locals, next_var),
	}
}
