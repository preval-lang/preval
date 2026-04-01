use std::collections::HashMap;

use crate::ir::Declaration;
use crate::ir::Function;
use crate::ir::Module;
use crate::ir::Operation;
use crate::ir::Statement;
use crate::ir::error::IRErrorInfo;
use crate::ir::to_ir;
use crate::parser::expression::InfoExpr;

pub fn variable_declaration(
    name: String,
    value_expr: Box<InfoExpr>,
    function: &mut Function,
    block: &mut usize,
    module: &mut Module,
    store: Option<usize>,
    declarations: &HashMap<String, Declaration>,
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
        module,
        *value_expr,
        Some(new_var),
        declarations,
        locals,
        next_var,
    )?;
    locals.insert(name, Declaration::Variable(new_var));
    if let Some(store) = store {
        function.ir[*block].statements.push(Statement::Operation(
            Operation::LoadLocal { src: new_var },
            Some(store),
        ));
    }
    Ok(())
}
