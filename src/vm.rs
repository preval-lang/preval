use std::collections::HashSet;
use std::{collections::HashMap, fs, process::exit};

use serde::{Deserialize, Serialize};

use crate::value::Value;
use crate::{
    builtins::get_builtins,
    ir::{Block, Function, Module, Operation, Statement, Terminal},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RunResult {
    Concrete(Value),
    Partial(Vec<Block>, HashMap<usize, Option<Value>>),
    ConditionalPartial {
        condition: (Vec<Block>, HashMap<usize, Option<Value>>),
        then: Box<RunResult>,
        els: Box<RunResult>,
    },
    ThenElseJump(bool),
}

pub fn run(module: &Module, function: Function, args: Vec<Option<Value>>) -> RunResult {
    let mut vars = HashMap::new();
    for (idx, arg) in args.iter().enumerate() {
        vars.insert(idx, arg.clone());
    }
    evaluate(module, function.ir, &mut vars, 0)
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

    let mut no_delete: HashSet<usize> = HashSet::new();

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
                                no_delete.insert(*leftn);
                                no_delete.insert(*rightn);
                            }
                            None => panic!("Undefined variable in index"),
                            Some(Some(right)) => match (left, right) {
                                (Value::String(left), Value::Usize(right)) => {
                                    if let Some(store) = store {
                                        vars.insert(
                                            *store,
                                            Some(Value::Char(left.chars().nth(*right).unwrap())),
                                        );
                                    }
                                }
                                _ => todo!("more index ops"),
                            },
                        },
                    },
                    Operation::LoadLiteral(lit) => {
                        if let Some(store) = store {
                            vars.insert(*store, Some(lit.clone()));
                        }
                    }
                    Operation::Call { function, args } => {
                        if let Some(builtin) = get_builtins().get(&function[0]) {
                            builtin.call(vars, args, store, &mut out, stmt, &mut no_delete);
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
                                no_delete.insert(*store);
                                no_delete.insert(*src);
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
                Statement::Delete(var) => {
                    if no_delete.contains(var) {
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
                Some(Some(Value::Bool(b))) => {
                    if *b {
                        blocks[block].terminal = Terminal::Jump(then);
                        block = then;
                    } else {
                        blocks[block].terminal = Terminal::Jump(els);
                        block = els;
                    }
                }
                Some(Some(other)) => panic!("Wrong value in condition"),
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
                if let Some(Some(Value::Bool(cond))) = vars.get(&var) {
                    return RunResult::ThenElseJump(*cond);
                } else {
                    panic!("variable unknown even after second pass {var} OR of wrong type")
                }
            }
            ref a => panic!("Unknown {a:?}"),
        }
    }
}
