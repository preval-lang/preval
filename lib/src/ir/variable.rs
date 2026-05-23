use std::collections::HashMap;

use crate::ir::error::IRError;
use crate::ir::error::IRErrorInfo;

use crate::ir::{Declaration, Function, Module, Operation, Statement};
use crate::parser::typ::InfoTypeExpr;
use crate::typ::TypeExpr;

pub fn variable(
    name: InfoTypeExpr,
    idx: usize,
    function: &mut Function,
    block: &mut usize,
    module: &mut Module,
    store: Option<usize>,
    declarations: &HashMap<String, Declaration>,
    locals: &mut HashMap<String, Declaration>,
    _next_var: &mut usize,
) -> Result<(), IRErrorInfo> {
    if let Some(store) = store {
        match name.expr {
            TypeExpr::Name(name) if locals.contains_key(&name) => match locals[&name] {
                Declaration::Variable(v) => {
                    function.ir[*block].statements.push(Statement {
                        store: Some(store),
                        operation: Operation::LoadLocal { src: v },
                    });
                }
            },
            _ => {
                function.ir[*block].statements.push(Statement {
                    store: Some(store),
                    operation: Operation::LoadFunction(
                        module.instantiator.instantiate(&name, &vec![]),
                    ),
                });
            }
        }
    }
    Ok(())
}
