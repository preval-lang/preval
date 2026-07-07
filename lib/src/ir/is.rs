use crate::ir::{IRContext, Operation, Statement, local::local};

pub fn is<'a>(
	name: String,
	typ: usize,
	block: &mut usize,
	store: Option<usize>,
	context: &mut IRContext<'_, 'a>,
) {
	let checked_var = context.var();

	local(name, block, Some(checked_var), context);

	if let Some(store) = store {
		context.blocks[*block].statements.push(Statement {
			store: Some(store),
			operation: Operation::Is {
				value: checked_var,
				typ,
			},
		});
	}
}
