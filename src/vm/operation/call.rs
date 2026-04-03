use std::collections::HashMap;

use crate::{
    ir::{Callable, Module, Operation, Partial, Statement},
    value::Value,
    vm::RunResult,
};

pub fn call(
    function: Callable,
    args: Vec<usize>,
    store: Option<usize>,
    out: &mut Vec<Statement>,
    module: &Module,
    vars: &mut HashMap<usize, Option<Value>>,
) {
    let function_value = match function {
        Callable::Partial(function) => function,
        Callable::Var(function_var) => match vars.get(&function_var) {
            Some(None) => {
                out.push(Statement::Operation(
                    Operation::Call { function, args },
                    store,
                ));
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

    let mut arg_values = Vec::new();
    for arg_var in &args {
        arg_values.push(match vars.get(&arg_var) {
            Some(value) => value,
            None => panic!(
                "Undefined variable {} in function argument, vars is {vars:?}",
                arg_var
            ),
        });
    }

    match function_value.data.call(module, arg_values) {
        RunResult::Concrete(value) => {
            if let Some(store) = store {
                vars.insert(store, Some(value));
            }
        }
        RunResult::Partial(blocks, start_block) => {
            out.push(Statement::Operation(
                Operation::Call {
                    function: Callable::Partial(Value::new(Partial {
                        blocks,
                        start_block,
                    })),
                    args,
                },
                store,
            ));
            if let Some(store) = store {
                vars.insert(store, None);
            }
        }
        RunResult::Residualise => {
            out.push(Statement::Operation(
                Operation::Call {
                    function: Callable::Partial(function_value),
                    args,
                },
                store,
            ));
            if let Some(store) = store {
                vars.insert(store, None);
            }
        }
    }
}
