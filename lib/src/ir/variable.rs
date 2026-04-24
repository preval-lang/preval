use std::collections::HashMap;

use crate::ir::error::IRError;
use crate::ir::error::IRErrorInfo;

use crate::ir::{Declaration, Function, Module, Operation, Statement};

pub fn variable(
    name: String,
    idx: usize,
    function: &mut Function,
    block: &mut usize,
    module: &mut Module,
    store: Option<usize>,
    declarations: &HashMap<String, Declaration>,
    locals: &mut HashMap<String, Declaration>,
    next_var: &mut usize,
) -> Result<(), IRErrorInfo> {
    if let Some(store) = store {
        match locals.get(&name).or(declarations.get(&name)) {
            None => {
                return Err(IRErrorInfo {
                    idx,
                    error: IRError::SymbolUndefined(name),
                });
            }
            Some(Declaration::Variable(v)) => {
                function.ir[*block].statements.push(Statement::Operation(
                    Operation::LoadLocal { src: *v },
                    Some(store),
                ));
            }
            Some(Declaration::Constant) => {
                function.ir[*block].statements.push(Statement::Operation(
                    Operation::LoadConstant(name),
                    Some(store),
                ));
            }
            _ => {
                return Err(IRErrorInfo {
                    idx,
                    error: IRError::NotStorable(name),
                });
            }
        }
    }
    Ok(())
}
