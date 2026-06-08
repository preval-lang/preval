use crate::ir::error::IRErrorInfo;
use crate::ir::{Block, to_ir};
use crate::{
	ir::{Declaration, Operation, Statement},
	parser::expression::InfoExpr,
};
use std::collections::HashMap;

pub fn index<'a>(
	left: Box<InfoExpr<'a>>,
	right: Box<InfoExpr<'a>>,
	function: &mut Vec<Block>,
	block: &mut usize,
	store: Option<usize>,
	locals: &mut HashMap<String, Declaration>,
	next_var: &mut usize,
) -> Result<(), IRErrorInfo<'a>> {
	let left_var = {
		*next_var += 1;
		*next_var
	};
	to_ir(
		function,
		block,
		*left,
		Some(left_var),
		locals,
		next_var,
		false,
	)?;
	let right_var = {
		*next_var += 1;
		*next_var
	};
	to_ir(
		function,
		block,
		*right,
		Some(right_var),
		locals,
		next_var,
		false,
	)?;

	function[*block].statements.push(Statement {
		store,
		operation: Operation::Index(left_var, right_var),
	});

	Ok(())
}
