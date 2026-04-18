use std::collections::{HashMap, HashSet};

use crate::{
    ir::{Block, Callable, Module, Operation, Partial, Statement, Terminal},
    value::{Value, structure::Struct, typ::Type},
    vm::RunResult,
};

#[derive(Clone, Debug)]
pub enum Poison {
    Value,
    Fields(HashMap<String, Poison>),
}

pub fn remove_unused(
    module: &Module,
    blocks: &Vec<Block>,
    start_block: usize,
    mut poison_vars: HashMap<usize, Poison>,
) -> Vec<Block> {
    return blocks.clone();

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
                Statement::Operation(Operation::LoadLocal { src }, store) => {
                    // we don't have to think about unused variables making others used because the partial evaluator will already remove them
                    if let Some(store) = store {
                        if let Some(poison) = poison_vars.get(src) {
                            poison_vars.insert(*store, poison.clone());
                        }
                        used_vars.insert(*src);
                    }
                }
                Statement::Operation(Operation::InitializeStruct(_, fields), store) => {
                    if let Some(store) = store {
                        let mut pf = HashMap::new();
                        for field_name in fields.keys() {
                            if poison_vars.contains_key(&fields[field_name]) {
                                pf.insert(
                                    field_name.clone(),
                                    poison_vars[&fields[field_name]].clone(),
                                );
                            } else {
                                used_vars.insert(fields[field_name]);
                            }
                        }
                        poison_vars.insert(*store, Poison::Fields(pf));
                    }
                }
                Statement::Operation(Operation::Call { function, args }, _) => {
                    for arg_var in args {
                        used_vars.insert(*arg_var);
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
                Statement::Operation(Operation::Access(left, right), store) => {
                    if let Some(store) = store {
                        used_vars.insert(*left);
                        match poison_vars.get(left) {
                            None => {}
                            Some(Poison::Value) => panic!("Use of poisoned var as left of access"),
                            Some(Poison::Fields(poisoned_fields)) => {
                                if poisoned_fields.contains_key(right) {
                                    poison_vars.insert(*store, poisoned_fields[right].clone());
                                }
                            }
                        }
                    }
                }
                Statement::Operation(Operation::LoadLiteral(v), store) => {
                    if let Some(store) = store {
                        fn get_poison(v: &Value) -> Option<Poison> {
                            if v.data.should_poison() {
                                Some(Poison::Value)
                            } else if let Some(struc) = v.data.as_any().downcast_ref::<Struct>() {
                                let mut poison_fields = HashMap::new();
                                for (field_name, value) in &struc.fields {
                                    if let Some(value) = value {
                                        let poison = get_poison(value);
                                        if let Some(poison) = poison {
                                            poison_fields.insert(field_name.clone(), poison);
                                        }
                                    } else {
                                        panic!(
                                            "all values in literal residualised struct should be known"
                                        )
                                    }
                                }
                                Some(Poison::Fields(poison_fields))
                            } else {
                                None
                            }
                        }
                        let poison = get_poison(v);
                        if let Some(poison) = poison {
                            poison_vars.insert(*store, poison);
                        }
                    }
                }
                Statement::Operation(Operation::Phi { block_to_var }, store) => {
                    if let Some(store) = store {
                        for (_, var) in block_to_var {
                            used_vars.insert(*var);
                            if let Some(poison) = poison_vars.get(var) {
                                poison_vars.insert(*store, poison.clone());
                            }
                        }
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

    for (var, poison) in &poison_vars {
        match poison {
            Poison::Value => {
                if used_vars.contains(var) {
                    panic!("Use of poisoned var {var}")
                }
            }
            _ => {}
        }
    }

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
                                remove_unused(&module, blocks, *start_block, poison_vars.clone()),
                                *start_block,
                            ),
                            RunResult::Residualise => RunResult::Residualise,
                        },
                        els: match els {
                            RunResult::Concrete(v) => RunResult::Concrete(v.clone()),
                            RunResult::Partial(blocks, start_block) => RunResult::Partial(
                                remove_unused(&module, blocks, *start_block, poison_vars.clone()),
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
                        // if used_vars.contains(var) {
                        new_block.statements.push(statement.clone());
                        // }
                    }
                    Statement::Operation(Operation::InitializeStruct(_, _), Some(var)) => {
                        // if used_vars.contains(var) {
                        new_block.statements.push(statement.clone());
                        // }
                    }
                    Statement::Operation(Operation::Call { function, args }, store) => {
                        let mut poisoned_args = HashMap::new();
                        for (arg_idx, arg_var) in args.iter().enumerate() {
                            if let Some(poison) = poison_vars.get(arg_var) {
                                poisoned_args.insert(arg_idx, poison.clone());
                            }
                        }
                        new_block.statements.push(Statement::Operation(
                            Operation::Call {
                                function: match function {
                                    Callable::Var(v) => Callable::Var(*v),
                                    Callable::Partial(Partial {
                                        blocks,
                                        start_block,
                                    }) => Callable::Partial(Partial {
                                        start_block: *start_block,
                                        blocks: remove_unused(
                                            module,
                                            blocks,
                                            *start_block,
                                            poisoned_args,
                                        ),
                                    }),
                                },
                                args: args.clone(),
                            },
                            store.clone(),
                        ));
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
