use crate::ir::{IRContext, Operation, Statement};

pub fn local(
	name: String,
	block: &mut usize,
	store: Option<usize>,
	context: &mut IRContext<'_, '_>,
) {
	if let Some(store) = store {
		match context.locals[&name] {
			v => {
				context.blocks[*block].statements.push(Statement {
					store: Some(store),
					operation: Operation::LoadLocal { src: v },
				});
			}
		}
	}
}
