use std::collections::HashMap;

use crate::{
    ir::{Operation, Statement},
    typ::{Program, RuntimeTypeExpr, type_id},
    value::Value,
};

pub fn is(
    value: usize,
    typ: RuntimeTypeExpr,
    module: &mut Program,
    vars: &mut HashMap<usize, Option<Value>>,
    out: &mut Vec<Statement>,
    store: Option<usize>,
    generics: &[usize],
) {
    let type_n = module
        .instantiate_rt(&typ, generics)
        .expect("move this to compile time by specialising function body");
    if let Some(store) = store {
        if let Some(value) = &vars[&value] {
            if module.compatible(value.typ, type_n, 0).unwrap() {
                vars.insert(store, Some(Value::new(true, type_id::bool)));
            } else {
                vars.insert(store, Some(Value::new(false, type_id::bool)));
            }
        } else {
            vars.insert(store, None);
            out.push(Statement {
                store: Some(store),
                operation: Operation::Is { value, typ },
            });
        }
    }
}
