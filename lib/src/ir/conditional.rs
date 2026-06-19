use crate::ir::IRContext;
use crate::typ::type_id;
use crate::value::Value;
use crate::value::primitive::EmptyTuple;
use crate::{
	ir::{Block, Operation, Statement, Terminal, to_ir},
	parser::expression::InfoExpr,
};
use std::collections::HashMap;

pub fn conditional<'a>(
	cond: Box<InfoExpr<'a>>,
	then: Box<InfoExpr<'a>>,
	els: Option<Box<InfoExpr<'a>>>,
	block: &mut usize,
	store: Option<usize>,
	tail: bool,
	context: &mut IRContext<'_, 'a>,
) {
	let cond_var = context.var();
	to_ir(block, *cond, Some(cond_var), false, context);

	let then_block_n = context.blocks.len();
	let mut then_block_n_mut = context.blocks.len();
	let then_block_var = context.var();
	context.blocks.push(Block {
		statements: Vec::new(),
		terminal: Terminal::Jump(context.blocks.len() + 1 + if els.is_some() { 1 } else { 0 }),
	});
	to_ir(
		&mut then_block_n_mut,
		*then,
		Some(then_block_var),
		tail,
		context,
	);

	let else_block = if let Some(els) = els {
		let else_block_n = context.blocks.len();
		let mut else_block_n_mut = context.blocks.len();
		let else_block_var = context.var();
		context.blocks.push(Block {
			statements: Vec::new(),
			terminal: Terminal::Jump(context.blocks.len() + 1),
		});
		to_ir(
			&mut else_block_n_mut,
			*els,
			Some(else_block_var),
			tail,
			context,
		);
		Some((else_block_n, else_block_var))
	} else {
		None
	};

	let old_terminal = context.blocks[*block].terminal.clone();

	context.blocks[*block].terminal = Terminal::CondJump {
		cond: cond_var,
		then: then_block_n,
		els: else_block.map(|f| f.0).unwrap_or(context.blocks.len()),
	};
	*block = context.blocks.len();

	context.blocks.push(Block {
		statements: Vec::new(),
		terminal: old_terminal,
	});

	if let Some(store) = store {
		if let Some(else_block) = else_block {
			let mut block_to_var = HashMap::new();
			block_to_var.insert(else_block.0, else_block.1);
			block_to_var.insert(then_block_n, then_block_var);
			context.blocks[*block].statements.push(Statement {
				store: Some(store),
				operation: Operation::Phi { block_to_var },
			});
		} else {
			context.blocks[*block].statements.push(Statement {
				store: Some(store),
				operation: Operation::LoadLiteral(Value::new(EmptyTuple, type_id::empty_tuple)),
			});
		}
	}
}
