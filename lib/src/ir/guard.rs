use crate::ir::{IRContext, Operation, Statement};
use crate::{
	ir::{Block, Terminal, to_ir},
	parser::expression::InfoExpr,
};

pub fn guard<'a>(
	dependency: Box<InfoExpr<'a>>,
	body: Box<InfoExpr<'a>>,
	block: &mut usize,
	store: Option<usize>,
	tail: bool,
	context: &mut IRContext<'_, 'a>,
) {
	let dep_var = context.var();
	to_ir(block, *dependency, Some(dep_var), false, context);

	let body_block = context.blocks.len();
	let mut body_block_mut = body_block;
	let continuation_block = body_block + 1;

	context.blocks.push(Block {
		statements: Vec::new(),
		terminal: Terminal::Jump(continuation_block),
	});

	let old_terminal = context.blocks[*block].terminal.clone();

	context.blocks.push(Block {
		statements: Vec::new(),
		terminal: old_terminal,
	});

	to_ir(&mut body_block_mut, *body, store, tail, context);

	context.blocks[*block].terminal = Terminal::Guard {
		dependency: dep_var,
		body: body_block,
		continuation: continuation_block,
	};
	*block = continuation_block;

	if let Some(store) = store {
		context.blocks[*block].statements.push(Statement {
			store: Some(store),
			operation: Operation::GuardPhi {
				block: body_block,
				var: store,
			},
		});
	}
}
