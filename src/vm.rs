use std::collections::HashMap;

use crate::ir::{Block, Function, Module, Operation, Statement, Terminal};

pub fn run(module: &Module, function: &Function) -> Option<Vec<u8>> {
    let mut vars = HashMap::new();
    let block = run_block(module, &function.ir[0], &mut vars);
    if block.statements.len() == 0 {
        match block.terminal {
            Terminal::Evaluate(Some(var)) => vars.get(&var)?.clone(),
            Terminal::Return(Some(var)) => vars.get(&var)?.clone(),
            _ => None,
        }
    } else {
        None
    }
}

fn run_block(
    module: &Module,
    // function: &Function,
    block: &Block,
    vars: &mut HashMap<usize, Option<Vec<u8>>>,
) -> Block {
    let mut out: Vec<Statement> = Vec::new();

    for stmt in &block.statements {
        match stmt {
            Statement::Operation(op, store) => match op {
                Operation::LoadGlobal { src } => {
                    if let Some(store) = store {
                        vars.insert(*store, Some(module.constants[*src].1.clone()));
                    }
                }
                Operation::Call { function, args } => {
                    if function[0].as_str() == "print" {
                        if let Some(text) = vars[&args[0]].clone() {
                            println!("{}", String::from_utf8(text).unwrap());
                        } else {
                            out.push(stmt.clone());
                        }
                    } else if let Some(fun) = module.functions.get(&function[0]) {
                        let out = run(module, fun);
                        if let Some(store) = store {
                            vars.insert(*store, out);
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

    return Block {
        terminal: block.terminal.clone(),
        statements: out,
    };
}
