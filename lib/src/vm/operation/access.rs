use std::collections::HashMap;

use crate::{
    ir::{Operation, Statement},
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
            out.push(Statement::Operation(Operation::Access(left, right), store));
        }
        None => panic!("Undefined variable in left of index"),
        Some(Some(left)) => {
            let val = Value::new(right.clone());
            let v = left.data.index(&val);

            if let Some(store) = store {
                vars.insert(store, Some(v));
            }
        }
    }
}
