use std::{collections::HashMap, process::exit};

use crate::ir::{Block, Function, Module, Operation, Statement, Terminal};

pub type VarRepr = Vec<u8>;

#[derive(Debug, Clone)]
pub enum RunResult {
    Concrete(Vec<u8>),
    Partial(Vec<Block>, HashMap<usize, Option<Vec<u8>>>),
    ConditionalPartial {
        condition: (Vec<Block>, HashMap<usize, Option<Vec<u8>>>),
        then: Box<RunResult>,
        els: Box<RunResult>,
    },
    ThenElseJump(bool),
}

pub fn run(module: &Module, function: Function, args: Vec<Option<Vec<u8>>>) -> RunResult {
    let mut vars = HashMap::new();
    for (idx, arg) in args.iter().enumerate() {
        vars.insert(idx, arg.clone());
    }
    evaluate(module, function.ir, &mut vars, 0)
}

pub fn evaluate(
    module: &Module,
    mut blocks: Vec<Block>,
    vars: &mut HashMap<usize, Option<Vec<u8>>>,
    start_block: usize,
) -> RunResult {
    let mut out: Vec<Statement> = Vec::new();

    let mut block = start_block;
    let mut last_block = 0;

    loop {
        for stmt in &blocks[block].statements {
            match stmt {
                Statement::Operation(op, store) => match op {
                    Operation::LoadGlobal { src } => {
                        if let Some(store) = store {
                            vars.insert(*store, Some(module.constants[*src].clone()));
                        }
                    }
                    Operation::Call { function, args } => {
                        if function[0].as_str() == "print" {
                            if let Some(Some(_)) = vars.get(&args[0]).clone() {
                                if let Some(Some(message)) = vars.get(&args[1]).clone() {
                                    println!("{}", String::from_utf8(message.to_vec()).unwrap())
                                } else {
                                    out.push(stmt.clone());
                                }
                            } else {
                                out.push(stmt.clone());
                            }
                        } else if let Some(fun) = module.functions.get(&function[0]) {
                            match run(
                                module,
                                fun.clone(),
                                args.iter()
                                    .map(|arg| vars.get(arg).unwrap().clone())
                                    .collect(),
                            ) {
                                RunResult::Concrete(res) => {
                                    if let Some(store) = store {
                                        vars.insert(*store, Some(res));
                                    }
                                }
                                _ => {
                                    todo!(
                                        "Inline functions when they can't be completely evaluated"
                                    );
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
                                vars.insert(*store, None);
                            }
                        }
                    }
                    Operation::Phi { block_to_var } => {
                        if let Some(store) = store {
                            let var_num = block_to_var.get(&last_block).expect(&format!(
                                "Block {block} did not expect to be jumped into by {last_block}"
                            ));
                            vars.insert(*store, vars.get(var_num).expect("Phi evaluated to undefined variable, must have forgot to store the result of the block").clone());
                        }
                    }
                    a => todo!("Other ops {a:?}"),
                },
            }
        }

        blocks[block].statements = out;
        out = Vec::new();

        last_block = block;

        match blocks[block].terminal {
            Terminal::Evaluate(Some(var)) | Terminal::Return(Some(var)) => match vars.get(&var) {
                Some(Some(data)) => {
                    for (_, d) in vars.iter() {
                        if d.is_none() {
                            return RunResult::Partial(blocks, vars.clone());
                        }
                    }
                    return RunResult::Concrete(data.clone());
                }
                Some(None) => return RunResult::Partial(blocks, vars.clone()),
                None => panic!("returning undefined variable {var}"),
            },
            Terminal::Jump(b) => block = b,
            Terminal::CondJump { cond, then, els } => match vars.get(&cond) {
                Some(Some(cond)) => {
                    if cond.len() > 1 && cond[0] != 0 {
                        blocks[block].terminal = Terminal::Jump(then);
                        block = then;
                    } else {
                        blocks[block].terminal = Terminal::Jump(els);
                        block = els;
                    }
                }
                Some(None) => {
                    let mut then_vars = vars.clone();
                    let then_result = evaluate(module, blocks.clone(), &mut then_vars, then);

                    let mut else_vars = vars.clone();
                    let else_result = evaluate(module, blocks.clone(), &mut else_vars, els);

                    blocks[block].terminal = Terminal::ThenElseJump(cond);

                    return RunResult::ConditionalPartial {
                        condition: (blocks, vars.clone()),
                        then: Box::new(then_result),
                        els: Box::new(else_result),
                    };
                }
                None => panic!("conditional jump cond was an undefined variable"),
            },
            Terminal::ThenElseJump(var) => {
                if let Some(Some(var)) = vars.get(&var) {
                    return RunResult::ThenElseJump(var.len() > 1 && var[0] != 0);
                } else {
                    panic!("variable unknown even after second pass {var}")
                }
            }
            ref a => panic!("Unknown {a:?}"),
        }
    }
}
