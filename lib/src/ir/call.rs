use crate::ir::Callable;
use crate::ir::IRContext;
use crate::ir::Operation;
use crate::ir::Statement;
use crate::ir::Terminal;
use crate::ir::error::IRErrorInfo;
use crate::ir::to_ir;
use crate::parser::expression::InfoExpr;

pub fn call<'a>(
	callee: Box<InfoExpr<'a>>,
	args: Vec<InfoExpr<'a>>,
	block: &mut usize,
	store: Option<usize>,
	tail: bool,
	context: &mut IRContext<'_, 'a>,
) -> Result<(), IRErrorInfo<'a>> {
	let callee = *callee;

	let mut arg_indexes = Vec::new();
	for arg in args {
		let i = context.var();
		to_ir(block, arg, Some(i), false, context)?;
		arg_indexes.push(i);
	}

	let fn_var = context.var();
	to_ir(block, callee, Some(fn_var), false, context)?;

	if tail {
		context.blocks[*block].terminal = Terminal::TailCall {
			function: Callable::Var(fn_var),
			args: arg_indexes,
		}
	} else {
		context.blocks[*block].statements.push(Statement {
			store,
			operation: Operation::Call {
				function: Callable::Var(fn_var),
				args: arg_indexes,
			},
		});
	}
	Ok(())
}
