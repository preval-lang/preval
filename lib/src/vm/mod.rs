mod operation;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    ir::{Block, Module, Operation, Statement, Terminal},
    value::{
        Value,
        structure::Struct,
        typ::{Poison, Type},
    },
    vm::operation::{access, call, index, initialize_struct, load_local, phi},
};

#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RunResult {
    Concrete(Value),
    Partial(Vec<Block>, usize),
    Residualise, // Native functions only! Because all preval functions can be partially evaluated even if there are no known arguments
}

pub fn evaluate(
    module: &Module,
    mut blocks: Vec<Block>,
    vars: &mut HashMap<usize, Option<Value>>,
    start_block: usize,
) -> RunResult {
    let mut last_block_num = start_block;
    let mut block_num = start_block;

    loop {
        let mut out: Vec<Statement> = Vec::new();

        let old_vars: Vec<_> = vars.keys().cloned().collect();

        for stmt in blocks[block_num].statements.clone() {
            match stmt {
                Statement::Delete(_) => todo!("Add delete statements"),
                Statement::Operation(Operation::Call { function, args }, store) => {
                    call(function, args, store, &mut out, module, vars)
                }
                Statement::Operation(Operation::LoadLiteral(value), store) => {
                    if let Some(store) = store {
                        vars.insert(store, Some(value.clone()));
                    }
                }
                Statement::Operation(Operation::LoadLocal { src }, store) => {
                    load_local(src, store, &mut out, vars);
                }
                Statement::Operation(Operation::Index(left, right), store) => {
                    index(left, right, store, &mut out, vars);
                }
                Statement::Operation(Operation::Phi { block_to_var }, store) => {
                    phi(block_to_var, store, last_block_num, &mut out, vars);
                }
                Statement::Operation(Operation::InitializeStruct(name, fields), store) => {
                    initialize_struct(name, fields, store, module, &mut out, vars);
                }
                Statement::Operation(Operation::Access(left, right), store) => {
                    access(left, right, store, &mut out, vars);
                }
            }
        }

        let new_vars: Vec<_> = vars.keys().filter(|k| !old_vars.contains(k)).collect();

        let residualise = out.len() != 0;

        for var_num in new_vars {
            if let Some(Some(var)) = vars.get(var_num) {
                if let Some(struc) = var.data.as_any().downcast_ref::<Struct>() {
                    let mut complete = true;
                    for field in &struc.fields {
                        if field.1.is_none() {
                            complete = false;
                            break;
                        }
                    }
                    if complete {
                        out.insert(
                            0,
                            Statement::Operation(
                                Operation::LoadLiteral(var.clone()),
                                Some(*var_num),
                            ),
                        );
                    }
                } else {
                    out.insert(
                        0,
                        Statement::Operation(Operation::LoadLiteral(var.clone()), Some(*var_num)),
                    );
                }
            }
        }

        match blocks[block_num].terminal.clone() {
            Terminal::CondJump { cond, then, els } => match vars.get(&cond) {
                Some(Some(value)) => {
                    if let Some(cond_bool) = value.data.as_any().downcast_ref::<bool>() {
                        let next_block = if *cond_bool { then } else { els };

                        blocks[block_num] = Block {
                            statements: out,
                            terminal: Terminal::Jump(next_block),
                        };
                        last_block_num = block_num;
                        block_num = next_block;
                    } else {
                        panic!("Non-bool condition")
                    }
                }
                Some(None) => {
                    blocks[block_num] = Block {
                        statements: out,
                        terminal: Terminal::Branch {
                            cond: cond,
                            then: evaluate(module, blocks.clone(), vars, then),
                            els: evaluate(module, blocks.clone(), vars, els),
                        },
                    };
                    return RunResult::Partial(blocks, start_block);
                }
                None => panic!("Undefined variable in condition"),
            },
            Terminal::Branch { cond, then, els } => match vars.get(&cond) {
                Some(Some(value)) => {
                    if let Some(cond_bool) = value.data.as_any().downcast_ref::<bool>() {
                        if *cond_bool {
                            return then.clone();
                        } else {
                            return els.clone();
                        }
                    } else {
                        panic!("Non-bool condition")
                    }
                }
                Some(None) => {
                    blocks[block_num] = Block {
                        statements: out,
                        terminal: Terminal::Branch {
                            cond: cond,
                            then: then.clone(),
                            els: els.clone(),
                        },
                    };
                    return RunResult::Partial(blocks, start_block);
                }
                None => panic!("Undefined variable in condition"),
            },
            Terminal::Jump(dest) => {
                blocks[block_num] = Block {
                    statements: out,
                    terminal: Terminal::Jump(dest),
                };
                last_block_num = block_num;
                block_num = dest;
            }
            Terminal::Return(var) => {
                blocks[block_num] = Block {
                    statements: out,
                    terminal: Terminal::Return(var),
                };
                if !residualise {
                    match vars.get(&var) {
                        Some(Some(var)) => {
                            return RunResult::Concrete(var.clone());
                        }
                        Some(None) => {
                            return RunResult::Partial(blocks, start_block);
                        }
                        None => panic!("Undefined variable in return"),
                    }
                }
                return RunResult::Partial(blocks, start_block);
            }
        }
    }
}
