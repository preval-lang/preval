use std::collections::HashMap;
use std::collections::HashSet;

use serde::Deserialize;
use serde::Serialize;

use crate::ir::Callable;
use crate::ir::Partial;
use crate::ir::{Block, Module, Operation, Statement, Terminal};
use crate::value::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RunResult {
    Concrete(Value),
    Partial(Vec<Block>, HashSet<usize>),
    Residualise,
}

pub fn evaluate(
    module: &Module,
    mut blocks: Vec<Block>,
    vars: &mut HashMap<usize, Option<Value>>,
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
                                let v = left.data.index(&right);
                                if let Some(store) = store {
                                    vars.insert(*store, Some(v));
                                }
                            }
                        },
                    },
                    Operation::LoadLiteral(lit) => {
                        if let Some(store) = store {
                            vars.insert(*store, Some(lit.clone()));
                        }
                    }
                    Operation::Call { function, args } => {
                        let funcvar = if let Callable::Var(v) = function {
                            Some(v)
                        } else {
                            None
                        };
                        let function = match function {
                            Callable::Partial(function) => &function,
                            Callable::Var(function) => match vars.get(function) {
                                Some(None) => {
                                    out.push(stmt.clone());
                                    for arg in args {
                                        resudual_vars.insert(*arg);
                                    }
                                    resudual_vars.insert(*function);
                                    continue;
                                }
                                None => panic!("Undefined variable in call"),
                                Some(Some(function)) => function,
                            },
                        };
                        let args_list = args
                            .iter()
                            .map(|v| vars.get(v).unwrap_or(&None))
                            .collect::<Vec<_>>();

                        match function.data.call(&module, args_list) {
                            RunResult::Concrete(v) => {
                                if let Some(store) = store {
                                    vars.insert(*store, Some(v));
                                }
                            }
                            RunResult::Residualise => {
                                out.push(stmt.clone());

                                for arg in args {
                                    resudual_vars.insert(*arg);
                                }
                                if let Some(store) = store {
                                    resudual_vars.insert(*store);
                                }
                                if let Some(funcvar) = funcvar {
                                    resudual_vars.insert(*funcvar);
                                }
                                if let Some(store) = store {
                                    vars.insert(*store, None);
                                }
                            }
                            RunResult::Partial(function, residuals) => {
                                out.push(Statement::Operation(
                                    Operation::Call {
                                        function: Callable::Partial(Value::new(Partial {
                                            blocks: function,
                                        })),
                                        args: args.clone(),
                                    },
                                    *store,
                                ));
                                if let Some(store) = store {
                                    vars.insert(*store, None);
                                }
                                for (i, arg) in args.iter().enumerate() {
                                    if residuals.contains(&i) {
                                        resudual_vars.insert(*arg);
                                    }
                                }
                                if let Some(store) = store {
                                    resudual_vars.insert(*store);
                                }
                            }
                        }
                    }

                    Operation::LoadLocal { src } => {
                        if let Some(store) = store {
                            if let Some(Some(data)) = vars.get(&src) {
                                vars.insert(*store, Some(data.clone()));
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
                                    Some(v) => Some(v.clone()),
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
                        return RunResult::Concrete(data.clone());
                    }

                    resudual_vars.insert(var);

                    let mut vars_statements: Vec<_> = resudual_vars
                        .iter()
                        .filter(|var| vars.get(var).unwrap().is_some())
                        .map(|var| {
                            Statement::Operation(
                                Operation::LoadLiteral(match vars.get(var) {
                                    Some(Some(v)) => v.clone(),
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
                                    Some(Some(v)) => v.clone(),
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
                                    Some(Some(v)) => v.clone(),
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
