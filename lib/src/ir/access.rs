use std::collections::HashMap;

use crate::ir::error::IRErrorInfo;
use crate::ir::to_ir;

use crate::{
    ir::{Declaration, Function, Module, Operation, Statement},
    parser::expression::InfoExpr,
};

pub fn access(
    left: Box<InfoExpr>,
    right: String,
    _idx: usize,
    function: &mut Function,
    block: &mut usize,
    module: &mut Module,
    store: Option<usize>,
    declarations: &HashMap<String, Declaration>,
    locals: &mut HashMap<String, Declaration>,
    next_var: &mut usize,
) -> Result<(), IRErrorInfo> {
    let left_var = {
        *next_var += 1;
        *next_var
    };
    to_ir(
        function,
        block,
        module,
        *left,
        Some(left_var),
        declarations,
        locals,
        next_var,
        false,
    )?;

    function.ir[*block].statements.push(Statement {
        store,
        operation: Operation::Access(left_var, right),
    });

    Ok(())
}
