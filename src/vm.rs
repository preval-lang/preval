use std::collections::HashSet;
use std::{collections::HashMap, fs, process::exit};

use serde::{Deserialize, Serialize};

use crate::ir::Callable;
use crate::value::Value;
use crate::{
    builtins::get_builtins,
    ir::{Block, Function, Module, Operation, Statement, Terminal},
};

#[derive(Debug, Clone)]
pub enum RunResult {
    Concrete(Box<dyn Value>),
    Partial(Vec<Block>, HashSet<usize>),
}

pub fn evaluate(
    module: &Module,
    mut blocks: Vec<Block>,
    vars: &mut HashMap<usize, Option<Box<dyn Value>>>,
    start_block: usize,
) -> RunResult {
    let mut out: Vec<Statement> = Vec::new();

    let mut block = start_block;
    let mut last_block = 0;

    let mut resudual_vars: HashSet<usize> = HashSet::new();

    loop {
        for stmt in &blocks[block].statements {
            match stmt {
                Statement::Operation(op, store) => match op {
                    Operation::Index(leftn, rightn) => match vars.get(leftn) {
                        Some(None) => {
                            out.push(stmt.clone());
                        }
                        None => panic!("Undefined variable in index"),
                        Some(Some(left)) => match vars.get(rightn) {
                            Some(None) => {
                                out.push(stmt.clone());
                                resudual_vars.insert(*leftn);
                                resudual_vars.insert(*rightn);
                            }
                            None => panic!("Undefined variable in index"),
                            Some(Some(right)) => {
                                let v = left.index(right.as_ref());
                                if let Some(store) = store {
                                    vars.insert(*store, Some(v));
                                }
                            }
                        },
                    },
                    Operation::LoadLiteral(lit) => {
                        if let Some(store) = store {
                            vars.insert(*store, Some(lit.as_ref().vclone()));
                        }
                    }
                    Operation::Call { function, args } => match function {
                        Callable::ModuleFunction(name) if get_builtins().contains_key(name) => {
                            get_builtins().get(name).unwrap().call(
                                vars,
                                args,
                                store,
                                &mut out,
                                stmt,
                                &mut resudual_vars,
                            );
                        }
                        function => {
                            let blocks = match function {
                                Callable::ModuleFunction(name) => module.functions[name].ir.clone(),
                                Callable::Partial(partial) => partial.clone(),
                            };
                            let mut args_map = HashMap::new();
                            for (i, arg) in args.iter().enumerate() {
                                args_map.insert(
                                    i,
                                    match vars.get(arg) {
                                        Some(Some(v)) => Some(v.as_ref().vclone()),
                                        _ => None,
                                    },
                                );
                            }
                            match evaluate(module, blocks, &mut args_map, 0) {
                                RunResult::Concrete(res) => {
                                    if let Some(store) = store {
                                        vars.insert(*store, Some(res));
                                    }
                                }
                                RunResult::Partial(func, residuals) => {
                                    if let Some(store) = store {
                                        resudual_vars.insert(*store);
                                    }
                                    for (i, arg) in args.iter().enumerate() {
                                        if residuals.contains(&i) {
                                            resudual_vars.insert(*arg);
                                        }
                                    }
                                    out.push(Statement::Operation(
                                        Operation::Call {
                                            function: Callable::Partial(func),
                                            args: args.clone(),
                                        },
                                        *store,
                                    ));
                                }
                            }
                        }
                    },
                    Operation::LoadLocal { src } => {
                        if let Some(store) = store {
                            if let Some(Some(data)) = vars.get(&src) {
                                vars.insert(*store, Some(data.as_ref().vclone()));
                            } else {
                                out.push(stmt.clone());
                                resudual_vars.insert(*store);
                                resudual_vars.insert(*src);
                                vars.insert(*store, None);
                            }
                        }
                    }
                    Operation::Phi { block_to_var } => {
                        if let Some(store) = store {
                            let var_num = block_to_var.get(&last_block).expect(&format!(
                                "Block {block} did not expect to be jumped into by {last_block}"
                            ));
                            let var = vars.get(var_num).expect("Phi evaluated to undefined variable, must have forgot to store the result of the block");

                            vars.insert(
                                *store,
                                match var {
                                    Some(v) => Some(v.as_ref().vclone()),
                                    None => None,
                                },
                            );
                        }
                    }
                },
                Statement::Delete(var) => {
                    if resudual_vars.contains(var) {
                        out.push(stmt.clone());
                    } else {
                        vars.remove(var);
                    }
                }
            }
        }

        blocks[block].statements = out;
        out = Vec::new();

        last_block = block;

        match blocks[block].terminal {
            Terminal::Evaluate(Some(var)) | Terminal::Return(Some(var)) => match vars.get(&var) {
                Some(Some(data)) => {
                    // TODO: Dedupe the partial returns
                    if blocks[block].statements.is_empty() {
                        return RunResult::Concrete(data.as_ref().vclone());
                    }

                    resudual_vars.insert(var);

                    let mut vars_statements: Vec<_> = resudual_vars
                        .iter()
                        .filter(|var| vars.get(var).unwrap().is_some())
                        .map(|var| {
                            Statement::Operation(
                                Operation::LoadLiteral(match vars.get(var) {
                                    Some(Some(v)) => v.as_ref().vclone(),
                                    _ => panic!("Residualise nonexistent variable"),
                                }),
                                Some(*var),
                            )
                        })
                        .collect();
                    vars_statements.extend_from_slice(&blocks[block].statements);
                    blocks[start_block].statements = vars_statements;
                    return RunResult::Partial(blocks, resudual_vars);
                }
                Some(None) => {
                    resudual_vars.insert(var);
                    let mut vars_statements: Vec<_> = resudual_vars
                        .iter()
                        .filter(|var| vars.get(var).unwrap().is_some())
                        .map(|var| {
                            Statement::Operation(
                                Operation::LoadLiteral(match vars.get(var) {
                                    Some(Some(v)) => v.as_ref().vclone(),
                                    _ => panic!("Residualise nonexistent variable"),
                                }),
                                Some(*var),
                            )
                        })
                        .collect();
                    vars_statements.extend_from_slice(&blocks[block].statements);
                    blocks[start_block].statements = vars_statements;
                    return RunResult::Partial(blocks, resudual_vars);
                }
                None => panic!("returning undefined variable {var}"),
            },
            Terminal::Jump(b) => block = b,
            Terminal::CondJump { cond, then, els } => match vars.get(&cond) {
                _ => todo!("downcast bools"),
                // Some(Some(Value::Bool(b))) => {
                //     if *b {
                //         blocks[block].terminal = Terminal::Jump(then);
                //         block = then;
                //     } else {
                //         blocks[block].terminal = Terminal::Jump(els);
                //         block = els;
                //     }
                // }
                Some(Some(other)) => panic!("Wrong value in condition"),
                Some(None) => {
                    resudual_vars.insert(cond);
                    let mut vars_statements: Vec<_> = resudual_vars
                        .iter()
                        .filter(|var| vars.get(var).unwrap().is_some())
                        .map(|var| {
                            Statement::Operation(
                                Operation::LoadLiteral(match vars.get(var) {
                                    Some(Some(v)) => v.as_ref().vclone(),
                                    _ => panic!("Residualise nonexistent variable"),
                                }),
                                Some(*var),
                            )
                        })
                        .collect();
                    vars_statements.extend_from_slice(&blocks[block].statements);
                    blocks[start_block].statements = vars_statements;
                    return RunResult::Partial(blocks, resudual_vars);
                }
                None => panic!("conditional jump cond was an undefined variable"),
            },
            ref a => panic!("Unknown {a:?}"),
        }
    }
}
