use std::collections::HashMap;

use crate::{
	ir::{IRContext, Operation, Statement, error::IRErrorInfo, to_ir},
	parser::{expression::InfoExpr, typ::InfoTypeExpr},
};

pub fn initialize_struct<'a>(
	typ: InfoTypeExpr<'a>,
	fields: HashMap<String, InfoExpr<'a>>,
	block: &mut usize,
	store: Option<usize>,
	context: &mut IRContext<'_, 'a>,
) -> Result<(), IRErrorInfo<'a>> {
	if let Some(store) = store {
		let mut field_vars: HashMap<String, usize> = HashMap::new();
		for (field_name, field_expr) in fields {
			let field_var = context.var();
			field_vars.insert(field_name, field_var);

			to_ir(block, field_expr, Some(field_var), false, context)?;
		}
		context.blocks[*block].statements.push(Statement {
			store: Some(store),
			operation: Operation::InitializeStruct(
				context
					.ins
					.instantiate(&typ, context.generics, context.prefix)
					.expect("Pass type error up properly"),
				field_vars,
			),
		});
	}
	Ok(())
}
