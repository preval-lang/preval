use std::collections::HashMap;

use crate::{
	ir::{Operation, Statement},
	value::Value,
};

pub fn phi(
	block_to_var: HashMap<usize, usize>,
	store: Option<usize>,
	last_block_num: usize,
	out: &mut Vec<Statement>,
	vars: &mut HashMap<usize, Option<Value>>,
) {
	if let Some(store) = store {
		let var_num = block_to_var.get(&last_block_num).expect(&format!(
			"Block did not expect to be jumped into by {last_block_num}"
		));
		let var = vars.get(var_num).expect(
			"Phi evaluated to undefined variable, must have forgot to store the result of the block",
		);

		if var.is_none() {
			out.push(Statement {
				store: Some(store),
				operation: Operation::Phi { block_to_var },
			});
		}

		vars.insert(
			store,
			match var {
				Some(v) => Some(v.clone()),
				None => None,
			},
		);
	}
}
