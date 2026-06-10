use crate::{
	error::Span,
	ir::{IRContext, Operation, Statement, error::IRErrorInfo, variable::variable},
	parser::typ::InfoTypeExpr,
	typ::TypeExpr,
};

pub fn is<'a>(
	name: String,
	typ: InfoTypeExpr<'a>,
	idx: Span<'a>,
	block: &mut usize,
	store: Option<usize>,
	context: &mut IRContext<'_, 'a>,
) -> Result<(), IRErrorInfo<'a>> {
	let checked_var = context.var();

	variable(
		InfoTypeExpr {
			expr: TypeExpr::Name(vec![name], false),
			idx: idx.clone(),
		},
		block,
		Some(checked_var),
		context,
	)?;

	if let Some(store) = store {
		context.blocks[*block].statements.push(Statement {
			store: Some(store),
			operation: Operation::Is {
				value: checked_var,
				typ: context
					.ins
					.instantiate(&typ, context.generics, context.prefix)
					.expect("Pass type errors up as IRErrors"),
			},
		});
	}

	Ok(())
}
