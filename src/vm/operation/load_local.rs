use std::collections::HashMap;

use crate::{
    ir::{Operation, Statement},
    value::Value,
};

pub fn load_local(
    src: usize,
    store: Option<usize>,
    out: &mut Vec<Statement>,
    vars: &mut HashMap<usize, Option<Value>>,
) {
    if let Some(store) = store {
        out.push(Statement::Operation(
            Operation::LoadLocal { src },
            Some(store),
        ));
        match vars.get(&src) {
            Some(Some(value)) => {
                vars.insert(store, Some(value.clone()));
            }
            Some(None) => {
                vars.insert(store, None);
            }
            None => {
                panic!("Load undefined local variable {src}");
            }
        }
    }
}
