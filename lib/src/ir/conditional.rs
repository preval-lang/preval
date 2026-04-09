use std::collections::HashMap;

use crate::ir::error::{IRError, IRErrorInfo};
use crate::{
    ir::{Block, Declaration, Function, Module, Operation, Statement, Terminal, to_ir},
    parser::expression::InfoExpr,
};

pub fn conditional(
    cond: Box<InfoExpr>,
    then: Box<InfoExpr>,
    els: Option<Box<InfoExpr>>,
    idx: usize,
    function: &mut Function,
    block: &mut usize,
    module: &mut Module,
    store: Option<usize>,
    declarations: &HashMap<String, Declaration>,
    locals: &mut HashMap<String, Declaration>,
    next_var: &mut usize,
) -> Result<(), IRErrorInfo> {
    let cond_var = {
        *next_var += 1;
        *next_var
    };
    to_ir(
        function,
        block,
        module,
        *cond,
        Some(cond_var),
        declarations,
        locals,
        next_var,
    )?;

    let then_block_n = function.ir.len();
    let mut then_block_n_mut = function.ir.len();
    let then_block_var = {
        *next_var += 1;
        *next_var
    };
    function.ir.push(Block {
        statements: Vec::new(),
        terminal: Terminal::Jump(function.ir.len() + 1 + if els.is_some() { 1 } else { 0 }),
    });
    to_ir(
        function,
        &mut then_block_n_mut,
        module,
        *then,
        Some(then_block_var),
        declarations,
        locals,
        next_var,
    )?;

    let else_block = if let Some(els) = els {
        let else_block_n = function.ir.len();
        let mut else_block_n_mut = function.ir.len();
        let else_block_var = {
            *next_var += 1;
            *next_var
        };
        function.ir.push(Block {
            statements: Vec::new(),
            terminal: Terminal::Jump(function.ir.len() + 1),
        });
        to_ir(
            function,
            &mut else_block_n_mut,
            module,
            *els,
            Some(else_block_var),
            declarations,
            locals,
            next_var,
        )?;
        Some((else_block_n, else_block_var))
    } else {
        None
    };

    let old_terminal = function.ir[*block].terminal.clone();

    function.ir[*block].terminal = Terminal::CondJump {
        cond: cond_var,
        then: then_block_n,
        els: else_block.map(|f| f.0).unwrap_or(function.ir.len()),
    };
    *block = function.ir.len();

    function.ir.push(Block {
        statements: Vec::new(),
        terminal: old_terminal,
    });

    if let Some(store) = store {
        if let Some(else_block) = else_block {
            let mut block_to_var = HashMap::new();
            block_to_var.insert(else_block.0, else_block.1);
            block_to_var.insert(then_block_n, then_block_var);
            function.ir[*block].statements.push(Statement::Operation(
                Operation::Phi { block_to_var },
                Some(store),
            ));
        } else {
            return Err(IRErrorInfo {
                idx,
                error: IRError::MissingElseBlock(),
            });
        }
    }

    Ok(())
}
