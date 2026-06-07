use crate::ir::Block;
use crate::ir::Callable;
use crate::ir::Declaration;
use crate::ir::Operation;
use crate::ir::Statement;
use crate::ir::Terminal;
use crate::ir::error::IRErrorInfo;
use crate::ir::to_ir;
use crate::parser::expression::InfoExpr;

use std::collections::HashMap;

pub fn call<'a>(
    callee: Box<InfoExpr<'a>>,
    args: Vec<InfoExpr<'a>>,
    function: &mut Vec<Block>,
    block: &mut usize,
    store: Option<usize>,
    locals: &mut HashMap<String, Declaration>,
    next_var: &mut usize,
    tail: bool,
) -> Result<(), IRErrorInfo<'a>> {
    let callee = *callee;

    let mut arg_indexes = Vec::new();
    for arg in args {
        let i = {
            *next_var += 1;
            *next_var
        };
        to_ir(function, block, arg, Some(i), locals, next_var, false)?;
        arg_indexes.push(i);
    }

    let fn_var = {
        *next_var += 1;
        *next_var
    };
    to_ir(
        function,
        block,
        callee,
        Some(fn_var),
        locals,
        next_var,
        false,
    )?;

    if tail {
        function[*block].terminal = Terminal::TailCall {
            function: Callable::Var(fn_var),
            args: arg_indexes,
        }
    } else {
        function[*block].statements.push(Statement {
            store,
            operation: Operation::Call {
                function: Callable::Var(fn_var),
                args: arg_indexes,
            },
        });
    }
    Ok(())
}
