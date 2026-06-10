use std::collections::HashMap;

use crate::{
	ir::{Operation, Statement},
	typ::Type,
	value::Value,
};

pub fn is(
	value: usize,
	typ: usize,
	_module: &mut Vec<Type>,
	vars: &mut HashMap<usize, Option<Value>>,
	out: &mut Vec<Statement>,
	store: Option<usize>,
) {
	if let Some(store) = store {
		if let Some(_value) = &vars[&value] {
			// if module.compatible(value.typ, typ, 0).unwrap() {
			// 	vars.insert(store, Some(Value::new(true, type_id::bool)));
			// } else {
			// 	vars.insert(store, Some(Value::new(false, type_id::bool)));
			// }
			todo!("re-add is with module separated from type info")
		} else {
			vars.insert(store, None);
			out.push(Statement {
				store: Some(store),
				operation: Operation::Is { value, typ },
			});
		}
	}
}
