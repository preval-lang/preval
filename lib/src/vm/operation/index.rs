use std::collections::HashMap;

use crate::{
    ir::{Operation, Statement},
    value::Value,
};

pub fn index(
    leftn: usize,
    rightn: usize,
    store: Option<usize>,
    out: &mut Vec<Statement>,
    vars: &mut HashMap<usize, Option<Value>>,
) {
    let r = vars.get(&rightn).cloned();
    match vars.get_mut(&leftn) {
        Some(None) => {
            if let Some(store) = store {
                vars.insert(store, None);
            }
            out.push(Statement::Operation(Operation::Index(leftn, rightn), store));
        }
        None => panic!("Undefined variable in left of index"),
        Some(Some(left)) => match r {
            Some(None) => {
                out.push(Statement::Operation(Operation::Index(leftn, rightn), store));
            }
            None => panic!("Undefined variable in left of index"),
            Some(Some(right)) => {
                let v = left.data.index(&right);

                if let Some(store) = store {
                    vars.insert(store, Some(v));
                }
            }
        },
    }
}
