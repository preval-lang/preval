use std::collections::HashMap;

use crate::ir::error::IRErrorInfo;
use crate::ir::{Block, to_ir};

use crate::{
    ir::{Declaration, Operation, Statement},
    parser::expression::InfoExpr,
};

pub fn access<'a>(
    left: Box<InfoExpr<'a>>,
    right: String,
    function: &mut Vec<Block>,
    block: &mut usize,
    store: Option<usize>,
    locals: &mut HashMap<String, Declaration>,
    next_var: &mut usize,
) -> Result<(), IRErrorInfo<'a>> {
    let left_var = {
        *next_var += 1;
        *next_var
    };
    to_ir(
        function,
        block,
        *left,
        Some(left_var),
        locals,
        next_var,
        false,
    )?;

    function[*block].statements.push(Statement {
        store,
        operation: Operation::Access(left_var, right),
    });

    Ok(())
}
