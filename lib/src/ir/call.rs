use crate::ir::Callable;
use crate::ir::Declaration;
use crate::ir::Function;
use crate::ir::Module;
use crate::ir::Operation;
use crate::ir::Statement;
use crate::ir::error::IRErrorInfo;
use crate::ir::to_ir;
use crate::parser::expression::InfoExpr;

use std::collections::HashMap;

pub fn call(
    callee: Box<InfoExpr>,
    args: Vec<InfoExpr>,
    function: &mut Function,
    block: &mut usize,
    module: &mut Module,
    store: Option<usize>,
    declarations: &HashMap<String, Declaration>,
    locals: &mut HashMap<String, Declaration>,
    next_var: &mut usize,
) -> Result<(), IRErrorInfo> {
    let callee = *callee;

    let mut arg_indexes = Vec::new();
    for arg in args {
        let i = {
            *next_var += 1;
            *next_var
        };
        to_ir(
            function,
            block,
            module,
            arg,
            Some(i),
            declarations,
            locals,
            next_var,
        )?;
        arg_indexes.push(i);
    }

    let fn_var = {
        *next_var += 1;
        *next_var
    };
    to_ir(
        function,
        block,
        module,
        callee,
        Some(fn_var),
        declarations,
        locals,
        next_var,
    )?;

    function.ir[*block].statements.push(Statement::Operation(
        Operation::Call {
            function: Callable::Var(fn_var),
            args: arg_indexes,
        },
        store,
    ));
    Ok(())
}
