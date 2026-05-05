use std::collections::HashMap;

use crate::{
    ir::{Callable, Module, Operation, Partial, Statement},
    typ::{Instantiator, type_id},
    value::Value,
    vm::RunResult,
};

pub fn call(
    function: Callable,
    args: Vec<usize>,
    store: Option<usize>,
    out: &mut Vec<Statement>,
    module: &mut Module,
    vars: &mut HashMap<usize, Option<Value>>,
) {
    let mut function_value = match &function {
        Callable::Partial(function) => Value::new(function.clone(), 0), // the type ID isn't used - this is a hack
        Callable::Var(function_var) => match vars.get(&function_var) {
            Some(None) => {
                out.push(Statement {
                    store,
                    operation: Operation::Call { function, args },
                });
                if let Some(store) = store {
                    vars.insert(store, None);
                }
                return;
            }
            None => panic!(
                "Undefined variable {} in call, vars is {vars:?}",
                function_var
            ),
            Some(Some(function)) => function.clone(),
        },
    };

    match function_value.data.call(module, prepare_args(&args, vars)) {
        RunResult::Concrete(value) => {
            if let Some(store) = store {
                vars.insert(store, Some(value));
            }
        }
        RunResult::Partial(blocks, start_block) => {
            out.push(Statement {
                store,
                operation: Operation::Call {
                    function: Callable::Partial(Partial {
                        blocks,
                        start_block,
                    }),
                    args,
                },
            });
            if let Some(store) = store {
                vars.insert(store, None);
            }
        }
        RunResult::Residualise => {
            out.push(Statement {
                store,
                operation: Operation::Call { function, args },
            });
            if let Some(store) = store {
                vars.insert(store, None);
            }
        }
    }
}

pub fn prepare_args<'a>(
    args: &Vec<usize>,
    vars: &'a mut HashMap<usize, Option<Value>>,
) -> Vec<&'a Option<Value>> {
    let mut arg_values = Vec::new();
    for arg_var in args {
        arg_values.push(match vars.get(&arg_var) {
            Some(value) => value,
            None => panic!(
                "Undefined variable {} in function argument, vars is {vars:?}",
                arg_var
            ),
        });
    }
    arg_values
}
