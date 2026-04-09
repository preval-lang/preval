use std::collections::{HashMap, HashSet};

use crate::{
    ir::{Block, Callable, Operation, Statement, Terminal},
    vm::RunResult,
};

pub fn remove_unused(blocks: &Vec<Block>, start_block: usize) -> Vec<Block> {
    let mut used_vars = HashSet::new();
    let mut used_blocks = HashSet::new();

    let mut block_queue = Vec::new();
    block_queue.push(start_block);

    loop {
        let block = if let Some(block) = block_queue.pop() {
            block
        } else {
            break;
        };

        used_blocks.insert(block);

        for stmt in &blocks[block].statements {
            match stmt {
                Statement::Delete(_) => todo!("Remove delete statement"),
                Statement::Operation(Operation::LoadLocal { src }, _) => {
                    // we don't have to think about unused variables making others used because the partial evaluator will already remove them
                    used_vars.insert(*src);
                }
                Statement::Operation(Operation::Call { function, args }, _) => {
                    for arg in args {
                        used_vars.insert(*arg);
                    }
                    match function {
                        Callable::Var(v) => {
                            used_vars.insert(*v);
                        }
                        Callable::Partial(_) => {}
                    }
                }
                Statement::Operation(Operation::Index(left, right), _) => {
                    used_vars.insert(*left);
                    used_vars.insert(*right);
                }
                Statement::Operation(Operation::LoadLiteral(_), _) => {}
                Statement::Operation(Operation::Phi { block_to_var }, _) => {
                    for (_, var) in block_to_var {
                        used_vars.insert(*var);
                    }
                }
            }
        }

        match &blocks[block].terminal {
            Terminal::Branch {
                cond,
                then: _,
                els: _,
            } => {
                used_vars.insert(*cond);
                // we will remove unused variables from the then and els branches when we're constructing the new block list
            }
            Terminal::Jump(new_block) => {
                block_queue.push(*new_block);
            }
            Terminal::Return(var) => {
                used_vars.insert(*var);
                break;
            }
            Terminal::CondJump { cond, then, els } => {
                used_vars.insert(*cond);
                block_queue.push(*then);
                block_queue.push(*els);
            }
        }
    }

    let mut out = Vec::new();

    for (block_num, block) in blocks.iter().enumerate() {
        if !used_blocks.contains(&block_num) {
            out.push(Block {
                statements: Vec::new(),
                terminal: Terminal::Jump(99999),
            });
        } else {
            let mut new_block = Block {
                statements: Vec::new(),
                terminal: match &block.terminal {
                    Terminal::Branch { cond, then, els } => Terminal::Branch {
                        cond: *cond,
                        then: match then {
                            RunResult::Concrete(v) => RunResult::Concrete(v.clone()),
                            RunResult::Partial(blocks, start_block) => RunResult::Partial(
                                remove_unused(blocks, *start_block),
                                *start_block,
                            ),
                            RunResult::Residualise => RunResult::Residualise,
                        },
                        els: match els {
                            RunResult::Concrete(v) => RunResult::Concrete(v.clone()),
                            RunResult::Partial(blocks, start_block) => RunResult::Partial(
                                remove_unused(blocks, *start_block),
                                *start_block,
                            ),
                            RunResult::Residualise => RunResult::Residualise,
                        },
                    },
                    other => other.clone(),
                },
            };

            for statement in &block.statements {
                match statement {
                    Statement::Operation(Operation::LoadLiteral(_), Some(var)) => {
                        if used_vars.contains(var) {
                            new_block.statements.push(statement.clone());
                        }
                    }
                    _ => {
                        new_block.statements.push(statement.clone());
                    }
                }
            }

            out.push(new_block);
        }
    }

    out
}
