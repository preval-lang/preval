use crate::ir::IRContext;
use crate::ir::{Operation, Statement};

pub fn global<'a>(
	id: usize,
	block: &mut usize,
	store: Option<usize>,
	context: &mut IRContext<'_, 'a>,
) {
	if let Some(store) = store {
		context.blocks[*block].statements.push(Statement {
			store: Some(store),
			operation: Operation::LoadFunction(id),
		});
	}
}
