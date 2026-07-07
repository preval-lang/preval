use crate::ir::{IRContext, to_ir};
use crate::ir::{Operation, Statement};
use crate::passes::type_check_expr::TypedExpr;

pub fn access<'a>(
	left: Box<TypedExpr>,
	right: String,
	block: &mut usize,
	store: Option<usize>,
	context: &mut IRContext<'_, 'a>,
) {
	let left_var = context.var();
	to_ir(block, *left, Some(left_var), false, context);

	context.blocks[*block].statements.push(Statement {
		store,
		operation: Operation::Access(left_var, right),
	});
}
