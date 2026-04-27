use std::collections::HashMap;

use crate::{
    ir::{Operation, Statement},
    value::Value,
};

pub fn guard_phi(
    block: usize,
    var: usize,
    store: Option<usize>,
    last_block_num: usize,
    out: &mut Vec<Statement>,
    vars: &mut HashMap<usize, Option<Value>>,
) {
    if let Some(store) = store {
        if last_block_num == block {
            vars.insert(store, vars[&var].clone())
        } else {
            out.push(Statement {
                store: Some(store),
                operation: Operation::GuardPhi { block, var },
            });
            vars.insert(store, None)
        };
    }
}
