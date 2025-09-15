use std::{collections::HashMap, process::exit};

use crate::ir::{Block, Function, Module, Operation, Statement, Terminal};

pub type VarRepr = Vec<u8>;

#[derive(Debug)]
pub enum RunResult {
    Concrete(Vec<u8>),
    Partial(Vec<Block<VarRepr>>, HashMap<usize, Option<Vec<u8>>>),
}

pub fn run(
    module: &Module<VarRepr>,
    function: Function<VarRepr>,
    args: Vec<Option<Vec<u8>>>,
) -> RunResult {
    let mut vars = HashMap::new();
    for (idx, arg) in args.iter().enumerate() {
        vars.insert(idx, arg.clone());
    }
    evaluate(module, function.ir, &mut vars)
}

pub fn evaluate(
    module: &Module<VarRepr>,
    mut blocks: Vec<Block<VarRepr>>,
    vars: &mut HashMap<usize, Option<Vec<u8>>>,
) -> RunResult {
    let mut out: Vec<Statement<VarRepr>> = Vec::new();

    let mut block = 0;
    let mut last_block = 0;

    loop {
        for stmt in &blocks[block].statements {
            match stmt {
                Statement::Operation(op, store) => match op {
                    Operation::LoadGlobal { src } => {
                        if let Some(store) = store {
                            vars.insert(*store, Some(module.constants[*src].1.clone()));
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
                                RunResult::Partial(blocks, variables) => {
                                    out.push(Statement::Operation(
                                        Operation::PartialCall { blocks, variables },
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
                None => panic!("returning undefined variable"),
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
                Some(None) => return RunResult::Partial(blocks, vars.clone()),
                None => panic!("conditional jump cond was an undefined variable"),
            },
            ref a => panic!("Unknown {a:?}"),
        }
    }
}
