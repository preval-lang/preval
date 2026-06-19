use crate::ir::IRContext;
use crate::ir::Operation;
use crate::ir::Statement;
use crate::ir::to_ir;
use crate::parser::expression::InfoExpr;

pub fn variable_declaration<'a>(
	name: String,
	value_expr: Box<InfoExpr<'a>>,
	block: &mut usize,
	store: Option<usize>,
	context: &mut IRContext<'_, 'a>,
) {
	let new_var = context.var();
	to_ir(block, *value_expr, Some(new_var), false, context);
	context.locals.insert(name, new_var);
	if let Some(store) = store {
		context.blocks[*block].statements.push(Statement {
			store: Some(store),
			operation: Operation::LoadLocal { src: new_var },
		});
	}
}
