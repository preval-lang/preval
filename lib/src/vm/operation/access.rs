use std::collections::HashMap;

use crate::{
	ir::{Operation, Statement},
	typ::type_id,
	value::Value,
};

pub fn access(
	left: usize,
	right: String,
	store: Option<usize>,
	out: &mut Vec<Statement>,
	vars: &mut HashMap<usize, Option<Value>>,
) {
	match vars.get_mut(&left) {
		Some(None) => {
			if let Some(store) = store {
				vars.insert(store, None);
			}
			out.push(Statement {
				store,
				operation: Operation::Access(left, right),
			});
		}
		None => panic!("Undefined variable in left of index"),
		Some(Some(left)) => {
			let val = Value::new(right.clone(), type_id::String);
			let v = left.data.index(&val);

			if let Some(store) = store {
				vars.insert(store, Some(v));
			}
		}
	}
}
