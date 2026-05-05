use std::collections::HashMap;

use crate::{
    ir::{
        Declaration, Function, Module, Operation, Statement, error::IRErrorInfo, variable::variable,
    },
    parser::typ::InfoTypeExpr,
    typ::TypeReference,
};

pub fn is(
    name: String,
    typ: TypeReference,
    idx: usize,
    function: &mut Function,
    block: &mut usize,
    module: &mut Module,
    store: Option<usize>,
    declarations: &HashMap<String, Declaration>,
    locals: &mut HashMap<String, Declaration>,
    next_var: &mut usize,
) -> Result<(), IRErrorInfo> {
    let checked_var = {
        *next_var += 1;
        *next_var
    };

    variable(
        name,
        idx,
        function,
        block,
        module,
        Some(checked_var),
        declarations,
        locals,
        next_var,
    )?;

    if let Some(store) = store {
        function.ir[*block].statements.push(Statement {
            store: Some(store),
            operation: Operation::Is {
                value: checked_var,
                typ,
            },
        });
    }

    Ok(())
}
