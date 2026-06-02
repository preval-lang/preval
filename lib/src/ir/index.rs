use std::collections::HashMap;

use crate::ir::error::IRErrorInfo;
use crate::ir::{Block, to_ir};

use crate::{
    ir::{Declaration, Module, Operation, Statement},
    parser::expression::InfoExpr,
};

pub fn index(
    left: Box<InfoExpr>,
    right: Box<InfoExpr>,
    function: &mut Vec<Block>,
    block: &mut usize,
    module: &mut Module,
    store: Option<usize>,
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
        locals,
        next_var,
        false,
    )?;
    let right_var = {
        *next_var += 1;
        *next_var
    };
    to_ir(
        function,
        block,
        module,
        *right,
        Some(right_var),
        locals,
        next_var,
        false,
    )?;

    function[*block].statements.push(Statement {
        store,
        operation: Operation::Index(left_var, right_var),
    });

    Ok(())
}
