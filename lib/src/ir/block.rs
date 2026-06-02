use crate::ir::Block;
use crate::ir::Declaration;
use crate::ir::Operation;
use crate::ir::Statement;
use crate::ir::error::IRErrorInfo;
use crate::ir::to_ir;
use crate::parser::expression::InfoExpr;
use crate::typ::type_id;
use crate::value::Value;
use crate::value::primitive::EmptyTuple;
use std::collections::HashMap;

pub fn compile_block(
    statements: Vec<InfoExpr>,
    returns: bool,
    function: &mut Vec<Block>,
    block: &mut usize,
    store: Option<usize>,
    locals: &mut HashMap<String, Declaration>,
    next_var: &mut usize,
    tail: bool,
) -> Result<(), IRErrorInfo> {
    let mut i = 0;
    let len = statements.len();
    for statement in statements {
        if i != len - 1 || !returns {
            to_ir(function, block, statement, None, locals, next_var, false)?;
        } else {
            to_ir(function, block, statement, store, locals, next_var, tail)?;
        }
        i += 1;
    }

    if (len == 0 || !returns) && store.is_some() {
        function[*block].statements.push(Statement {
            store,
            operation: Operation::LoadLiteral(Value::new(EmptyTuple {}, type_id::empty_tuple)),
        });
    }
    Ok(())
}
