use std::collections::HashMap;

use crate::{
	ir::{IRContext, Operation, Statement, to_ir},
	passes::type_check_expr::TypedExpr,
};

pub fn initialize_struct<'a>(
	typ: usize,
	fields: HashMap<String, TypedExpr>,
	block: &mut usize,
	store: Option<usize>,
	context: &mut IRContext<'_, 'a>,
) {
	if let Some(store) = store {
		let mut field_vars: HashMap<String, usize> = HashMap::new();
		for (field_name, field_expr) in fields {
			let field_var = context.var();
			field_vars.insert(field_name, field_var);

			to_ir(block, field_expr, Some(field_var), false, context);
		}
		context.blocks[*block].statements.push(Statement {
			store: Some(store),
			operation: Operation::InitializeStruct(typ, field_vars),
		});
	}
}
