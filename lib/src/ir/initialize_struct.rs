use std::collections::HashMap;

use crate::{
    ir::{Block, Declaration, Module, Operation, Statement, error::IRErrorInfo, to_ir},
    parser::{expression::InfoExpr, typ::InfoTypeExpr},
};

pub fn initialize_struct(
    typ: InfoTypeExpr,
    fields: HashMap<String, InfoExpr>,
    function: &mut Vec<Block>,
    block: &mut usize,
    module: &mut Module,
    store: Option<usize>,
    locals: &mut HashMap<String, Declaration>,
    next_var: &mut usize,
) -> Result<(), IRErrorInfo> {
    if let Some(store) = store {
        let mut field_vars: HashMap<String, usize> = HashMap::new();
        for (field_name, field_expr) in fields {
            let field_var = {
                *next_var += 1;
                *next_var
            };
            field_vars.insert(field_name, field_var);

            to_ir(
                function,
                block,
                module,
                field_expr,
                Some(field_var),
                locals,
                next_var,
                false,
            )?;
        }
        function[*block].statements.push(Statement {
            store: Some(store),
            operation: Operation::InitializeStruct(typ, field_vars),
        });
    }
    Ok(())
}
