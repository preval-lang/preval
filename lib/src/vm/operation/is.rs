use std::collections::HashMap;

use crate::{
    ir::{Module, Operation, Statement},
    parser::typ::InfoTypeExpr,
    typ::type_id,
    value::Value,
};

pub fn is(
    value: usize,
    typ: InfoTypeExpr,
    module: &mut Module,
    vars: &mut HashMap<usize, Option<Value>>,
    out: &mut Vec<Statement>,
    store: Option<usize>,
    generics: &[usize],
) {
    let type_n = module.instantiator.instantiate(&typ, generics);
    if let Some(store) = store {
        if let Some(value) = &vars[&value] {
            if module
                .instantiator
                .compatible(value.typ, type_n, 0)
                .unwrap()
            {
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
