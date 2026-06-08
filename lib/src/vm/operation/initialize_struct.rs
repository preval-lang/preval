use crate::typ::{Program, RuntimeTypeExpr};
use crate::value::Value;
use crate::value::structure::Struct;
use crate::vm::Statement;
use std::collections::HashMap;

pub fn initialize_struct(
	typ: RuntimeTypeExpr,
	fields: HashMap<String, usize>,
	store: Option<usize>,
	module: &mut Program,
	out: &mut Vec<Statement>,
	vars: &mut HashMap<usize, Option<Value>>,
	generics: &[usize],
) {
	if let Some(store) = store {
		let mut output_struct: HashMap<String, Option<Value>> = HashMap::new();

		let mut residualise = false;

		for (field_name, field_value) in &fields {
			let value = vars.get(field_value).unwrap_or(&None).clone();
			if value.is_none() {
				residualise = true;
			}
			output_struct.insert(field_name.clone(), value);
		}

		let type_n = module
			.instantiate_rt(&typ, generics)
			.expect("move this to compile time by specialising the function body");

		vars.insert(
			store,
			Some(Value::new(
				Struct {
					fields: output_struct,
				},
				type_n,
			)),
		);

		if residualise {
			out.push(Statement {
				store: Some(store),
				operation: crate::ir::Operation::InitializeStruct(typ, fields),
			});
		}
	}
}
