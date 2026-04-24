use std::collections::HashMap;

use crate::ir::error::IRErrorInfo;
use crate::ir::{Operation, Statement};
use crate::{
    ir::{Block, Declaration, Function, Module, Terminal, to_ir},
    parser::expression::InfoExpr,
};

pub fn guard(
    dependency: Box<InfoExpr>,
    body: Box<InfoExpr>,
    idx: usize,
    function: &mut Function,
    block: &mut usize,
    module: &mut Module,
    store: Option<usize>,
    declarations: &HashMap<String, Declaration>,
    locals: &mut HashMap<String, Declaration>,
    next_var: &mut usize,
) -> Result<(), IRErrorInfo> {
    let dep_var = {
        *next_var += 1;
        *next_var
    };
    to_ir(
        function,
        block,
        module,
        *dependency,
        Some(dep_var),
        declarations,
        locals,
        next_var,
    )?;

    let body_block = function.ir.len();
    let mut body_block_mut = function.ir.len();

    function.ir.push(Block {
        statements: Vec::new(),
        terminal: Terminal::Jump(function.ir.len() + 1),
    });

    to_ir(
        function,
        &mut body_block_mut,
        module,
        *body,
        store,
        declarations,
        locals,
        next_var,
    )?;

    let old_terminal = function.ir[*block].terminal.clone();

    function.ir[*block].terminal = Terminal::Guard {
        dependency: dep_var,
        body: body_block,
        continuation: function.ir.len(),
    };
    *block = function.ir.len();

    function.ir.push(Block {
        statements: Vec::new(),
        terminal: old_terminal,
    });

    if let Some(store) = store {
        function.ir[*block].statements.push(Statement::Operation(
            Operation::GuardPhi {
                block: body_block,
                var: store,
            },
            Some(store),
        ));
    }

    Ok(())
}
