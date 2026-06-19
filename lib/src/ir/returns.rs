use crate::ir::IRContext;
use crate::ir::Operation;
use crate::ir::Statement;
use crate::ir::Terminal;
use crate::ir::to_ir;
use crate::parser::expression::InfoExpr;
use crate::typ::type_id;
use crate::value::Value;
use crate::value::primitive::EmptyTuple;

pub fn returns<'a>(
	value_expr: Option<Box<InfoExpr<'a>>>,
	block: &mut usize,
	context: &mut IRContext<'_, 'a>,
) {
	let return_var = context.var();
	context.blocks[*block].terminal = Terminal::Return(if let Some(value_expr) = value_expr {
		to_ir(block, *value_expr, Some(return_var), true, context);
		return_var
	} else {
		context.blocks[*block].statements.push(Statement {
			store: Some(return_var),
			operation: Operation::LoadLiteral(Value::new(EmptyTuple, type_id::empty_tuple)),
		});
		return_var
	});
}
