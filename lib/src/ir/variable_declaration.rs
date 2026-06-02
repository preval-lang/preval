use std::collections::HashMap;

use crate::ir::Block;
use crate::ir::Declaration;
use crate::ir::Operation;
use crate::ir::Statement;
use crate::ir::error::IRErrorInfo;
use crate::ir::to_ir;
use crate::parser::expression::InfoExpr;

pub fn variable_declaration(
    name: String,
    value_expr: Box<InfoExpr>,
    function: &mut Vec<Block>,
    block: &mut usize,
    store: Option<usize>,
    locals: &mut HashMap<String, Declaration>,
    next_var: &mut usize,
) -> Result<(), IRErrorInfo> {
    let new_var = {
        *next_var += 1;
        *next_var
    };
    to_ir(
        function,
        block,
        *value_expr,
        Some(new_var),
        locals,
        next_var,
        false,
    )?;
    locals.insert(name, Declaration::Variable(new_var));
    if let Some(store) = store {
        function[*block].statements.push(Statement {
            store: Some(store),
            operation: Operation::LoadLocal { src: new_var },
        });
    }
    Ok(())
}
