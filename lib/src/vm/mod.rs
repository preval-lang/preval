mod operation;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    ir::{Block, Callable, Function, Module, Operation, Partial, Statement, Terminal},
    value::{Value, structure::Struct},
    vm::operation::{access, call, guard_phi, index, initialize_struct, load_local, phi},
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
                Statement {
                    store,
                    operation: Operation::GuardPhi { block, var },
                } => guard_phi(block, var, store, last_block_num, &mut out, vars),
                Statement {
                    store,
                    operation: Operation::Call { function, args },
                } => call(function, args, store, &mut out, module, vars),
                Statement {
                    store,
                    operation: Operation::LoadConstant(name),
                } => {
                    if let Some(value) = module.objects.get(&name) {
                        if let Some(store) = store {
                            vars.insert(store, Some(value.clone()));
                        }
                    }
                }
                Statement {
                    store,
                    operation: Operation::LoadLiteral(value),
                } => {
                    if let Some(store) = store {
                        vars.insert(store, Some(value.clone()));
                    }
                }
                Statement {
                    store,
                    operation: Operation::LoadLocal { src },
                } => {
                    load_local(src, store, &mut out, vars);
                }
                Statement {
                    store,
                    operation: Operation::Index(left, right),
                } => {
                    index(left, right, store, &mut out, vars);
                }
                Statement {
                    store,
                    operation: Operation::Phi { block_to_var },
                } => {
                    phi(block_to_var, store, last_block_num, &mut out, vars);
                }
                Statement {
                    store,
                    operation: Operation::InitializeStruct(name, fields),
                } => {
                    initialize_struct(name, fields, store, module, &mut out, vars);
                }
                Statement {
                    store,
                    operation: Operation::Access(left, right),
                } => {
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
                            Statement {
                                store: Some(*var_num),
                                operation: Operation::LoadLiteral(var.clone()),
                            },
                        );
                    }
                } else {
                    out.insert(
                        0,
                        Statement {
                            store: Some(*var_num),
                            operation: Operation::LoadLiteral(var.clone()),
                        },
                    );
                }
            }
        }

        match blocks[block_num].terminal.clone() {
            Terminal::Guard {
                dependency,
                body,
                continuation,
            } => match vars.get(&dependency) {
                Some(Some(_)) => {
                    blocks[block_num] = Block {
                        statements: out,
                        terminal: Terminal::Jump(body),
                    };
                    last_block_num = block_num;
                    block_num = body;
                }
                Some(None) => {
                    blocks[block_num] = Block {
                        statements: out,
                        terminal: Terminal::Jump(body),
                    };
                    last_block_num = block_num;
                    block_num = continuation;
                }
                None => panic!("undefined variable in guard"),
            },
            Terminal::TailCall { function, args } => {
                let mut callable_var = None;
                let ir: Option<(usize, Vec<Block>)> = match function {
                    Callable::Var(var) => {
                        callable_var = Some(var);
                        if let Some(value) = vars.get(&var) {
                            if let Some(value) = value {
                                if let Some(result) = value.data.as_any().downcast_ref::<Function>()
                                {
                                    Some((0, result.ir.clone()))
                                } else {
                                    match value
                                        .clone()
                                        .data
                                        .call(module, args.iter().map(|idx| &vars[idx]).collect())
                                    {
                                        RunResult::Concrete(return_value) => {
                                            if residualise {
                                                out.push(Statement {
                                                    store: { Some(90000) },
                                                    operation: Operation::LoadLiteral(return_value),
                                                });
                                                blocks[block_num] = Block {
                                                    statements: out,
                                                    terminal: Terminal::Return(90000),
                                                };
                                                return RunResult::Partial(blocks, start_block);
                                            } else {
                                                return RunResult::Concrete(return_value);
                                            }
                                        }
                                        RunResult::Partial(blocks, start_block) => {
                                            Some((start_block, blocks.clone()))
                                        }
                                        RunResult::Residualise => None,
                                    }
                                }
                            } else {
                                None
                            }
                        } else {
                            panic!("Undefined variable in tail call")
                        }
                    }
                    Callable::Partial(partial) => {
                        Some((partial.start_block, partial.blocks.clone()))
                    }
                };
                let (new_start_block, new_blocks) = if let Some(ir) = ir {
                    ir
                } else {
                    blocks[block_num] = Block {
                        statements: out,
                        terminal: Terminal::TailCall {
                            function: Callable::Var(callable_var.unwrap()),
                            args,
                        },
                    };

                    return RunResult::Partial(blocks, start_block);
                };

                let mut new_vars = HashMap::new();

                for (idx, arg) in args.iter().enumerate() {
                    new_vars.insert(idx, vars.get(&arg).expect("Defined variable").clone());
                }

                *vars = new_vars;

                if residualise {
                    match evaluate(module, new_blocks, vars, new_start_block) {
                        RunResult::Concrete(val) => {
                            out.push(Statement {
                                store: {
                                    println!("todo: use next var {}:{}", file!(), line!());
                                    Some(90000)
                                },
                                operation: Operation::LoadLiteral(val),
                            });
                            blocks[block_num] = Block {
                                statements: out,
                                terminal: Terminal::Return(90000),
                            };
                            return RunResult::Partial(blocks, start_block);
                        }
                        RunResult::Partial(new_blocks, new_start_block) => {
                            blocks[block_num] = Block {
                                statements: out,
                                terminal: Terminal::TailCall {
                                    function: Callable::Partial(Partial {
                                        blocks: new_blocks,
                                        start_block: new_start_block,
                                    }),
                                    args,
                                },
                            };
                        }
                        RunResult::Residualise => {
                            blocks[block_num].statements = out;
                            return RunResult::Partial(blocks, start_block);
                        }
                    }

                    return RunResult::Partial(blocks, start_block);
                } else {
                    blocks = new_blocks;
                    last_block_num = block_num;
                    block_num = new_start_block;
                }

                continue;
            }
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
