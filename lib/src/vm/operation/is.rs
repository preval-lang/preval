use std::collections::HashMap;

use crate::{
    ir::{Module, Operation, Statement},
    passes::old_type_check_expr::compatible,
    typ::{TypeExpr, type_names},
    value::Value,
};

pub fn is(
    value: usize,
    typ: TypeExpr,
    module: &mut Module,
    vars: &mut HashMap<usize, Option<Value>>,
    out: &mut Vec<Statement>,
    store: Option<usize>,
) {
    if let Some(store) = store {
        if let Some(value) = &vars[&value] {
            vars.insert(
                store,
                Some(Value::new(
                    compatible(&value.typ, &typ, module, false).unwrap(),
                    type_names::bool(),
                )),
            );
        } else {
            vars.insert(store, None);
            out.push(Statement {
                store: Some(store),
                operation: Operation::Is { value, typ },
            });
        }
    }
}
