use std::collections::HashMap;

use crate::ir::error::IRErrorInfo;
use crate::ir::{Operation, Statement};
use crate::{
    ir::{Block, Declaration, Terminal, to_ir},
    parser::expression::InfoExpr,
};

pub fn guard<'a>(
    dependency: Box<InfoExpr<'a>>,
    body: Box<InfoExpr<'a>>,
    function: &mut Vec<Block>,
    block: &mut usize,
    store: Option<usize>,
    locals: &mut HashMap<String, Declaration>,
    next_var: &mut usize,
    tail: bool,
) -> Result<(), IRErrorInfo<'a>> {
    let dep_var = {
        *next_var += 1;
        *next_var
    };
    to_ir(
        function,
        block,
        *dependency,
        Some(dep_var),
        locals,
        next_var,
        false,
    )?;

    let body_block = function.len();
    let mut body_block_mut = body_block;
    let continuation_block = body_block + 1;

    function.push(Block {
        statements: Vec::new(),
        terminal: Terminal::Jump(continuation_block),
    });

    let old_terminal = function[*block].terminal.clone();

    function.push(Block {
        statements: Vec::new(),
        terminal: old_terminal,
    });

    to_ir(
        function,
        &mut body_block_mut,
        *body,
        store,
        locals,
        next_var,
        tail,
    )?;

    function[*block].terminal = Terminal::Guard {
        dependency: dep_var,
        body: body_block,
        continuation: continuation_block,
    };
    *block = continuation_block;

    if let Some(store) = store {
        function[*block].statements.push(Statement {
            store: Some(store),
            operation: Operation::GuardPhi {
                block: body_block,
                var: store,
            },
        });
    }

    Ok(())
}
