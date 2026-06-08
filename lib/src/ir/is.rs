use std::collections::HashMap;

use crate::{
	error::Span,
	ir::{Block, Declaration, Operation, Statement, error::IRErrorInfo, variable::variable},
	parser::typ::InfoTypeExpr,
	typ::TypeExpr,
};

pub fn is<'a>(
	name: String,
	typ: InfoTypeExpr<'a>,
	idx: Span<'a>,
	function: &mut Vec<Block>,
	block: &mut usize,
	store: Option<usize>,
	locals: &mut HashMap<String, Declaration>,
	next_var: &mut usize,
) -> Result<(), IRErrorInfo<'a>> {
	let checked_var = {
		*next_var += 1;
		*next_var
	};

	variable(
		InfoTypeExpr {
			expr: TypeExpr::Name(vec![name]),
			idx: idx.clone(),
		},
		function,
		block,
		Some(checked_var),
		locals,
	)?;

	if let Some(store) = store {
		function[*block].statements.push(Statement {
			store: Some(store),
			operation: Operation::Is {
				value: checked_var,
				typ: typ.into(),
			},
		});
	}

	Ok(())
}
