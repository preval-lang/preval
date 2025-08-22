use std::collections::HashMap;

use crate::ir::{Block, Function, Module, Operation, Statement, Terminal};

pub type VarRepr = Vec<u8>;

#[derive(Debug)]
pub enum RunResult {
    Void,
    Concrete(Vec<u8>),
    Partial(Vec<Block<VarRepr>>, HashMap<usize, Option<Vec<u8>>>),
}

pub fn run(
    module: &Module<VarRepr>,
    function: &Function<VarRepr>,
    args: Vec<Option<Vec<u8>>>,
) -> RunResult {
    let mut vars = HashMap::new();
    for (idx, arg) in args.iter().enumerate() {
        vars.insert(idx, arg.clone());
    }
    let blocks = evaluate_function(module, &function, &mut vars);
    if blocks[0].statements.len() == 0 {
        match blocks[0].terminal {
            Terminal::Evaluate(Some(var)) | Terminal::Return(Some(var)) => match vars.get(&var) {
                Some(Some(var)) => RunResult::Concrete(var.clone()),
                Some(None) => unreachable!("Return uncomputable variable"),
                None => panic!("Undefined variable"),
            },
            _ => RunResult::Void,
        }
    } else {
        RunResult::Partial(blocks, vars)
    }
}

fn evaluate_function(
    module: &Module<VarRepr>,
    function: &Function<VarRepr>,
    vars: &mut HashMap<usize, Option<Vec<u8>>>,
) -> Vec<Block<VarRepr>> {
    let mut out: Vec<Statement<VarRepr>> = Vec::new();

    for stmt in &function.ir[0].statements {
        match stmt {
            Statement::Operation(op, store) => match op {
                Operation::LoadGlobal { src } => {
                    if let Some(store) = store {
                        vars.insert(*store, Some(module.constants[*src].1.clone()));
                    }
                }
                Operation::Call { function, args } => {
                    if function[0].as_str() == "print" {
                        if let Some(Some(text)) = vars.get(&args[0]).clone() {
                            println!("{}", String::from_utf8(text.to_vec()).unwrap());
                        } else {
                            out.push(stmt.clone());
                        }
                    } else if let Some(fun) = module.functions.get(&function[0]) {
                        match run(
                            module,
                            fun,
                            args.iter()
                                .map(|arg| vars.get(arg).unwrap().clone())
                                .collect(),
                        ) {
                            RunResult::Void => {}
                            RunResult::Concrete(res) => {
                                if let Some(store) = store {
                                    vars.insert(*store, Some(res));
                                }
                            }
                            RunResult::Partial(blocks, incomplete_vars) => {
                                out.push(Statement::Operation(
                                    Operation::PartialCall {
                                        function: blocks,
                                        variables: incomplete_vars,
                                    },
                                    *store,
                                ));
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
                        }
                    }
                }
                _ => todo!("Other ops"),
            },
        }
    }

    return vec![Block::<VarRepr> {
        terminal: function.ir[0].terminal.clone(),
        statements: out,
    }];
}
