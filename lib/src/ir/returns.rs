use std::collections::HashMap;

use crate::ir::Block;
use crate::ir::Declaration;
use crate::ir::Operation;
use crate::ir::Statement;
use crate::ir::Terminal;
use crate::ir::error::IRErrorInfo;
use crate::ir::to_ir;
use crate::parser::expression::InfoExpr;
use crate::typ::type_id;
use crate::value::Value;
use crate::value::primitive::EmptyTuple;

pub fn returns<'a>(
	value_expr: Option<Box<InfoExpr<'a>>>,
	function: &mut Vec<Block>,
	block: &mut usize,
	locals: &mut HashMap<String, Declaration>,
	next_var: &mut usize,
) -> Result<(), IRErrorInfo<'a>> {
	let return_var = {
		*next_var += 1;
		*next_var
	};
	function[*block].terminal = Terminal::Return(if let Some(value_expr) = value_expr {
		to_ir(
			function,
			block,
			*value_expr,
			Some(return_var),
			locals,
			next_var,
			true,
		)?;
		return_var
	} else {
		function[*block].statements.push(Statement {
			store: Some(return_var),
			operation: Operation::LoadLiteral(Value::new(EmptyTuple, type_id::empty_tuple)),
		});
		return_var
	});
	Ok(())
}
