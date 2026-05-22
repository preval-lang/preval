use crate::ir::Declaration;
use crate::ir::Function;
use crate::ir::Module;
use crate::ir::Operation;
use crate::ir::Statement;
use crate::ir::error::IRErrorInfo;
use crate::ir::to_ir;
use crate::parser::expression::InfoExpr;
use crate::typ::TypeExpr;
use crate::value::Value;
use crate::value::primitive::EmptyTuple;
use std::collections::HashMap;

pub fn compile_block(
    statements: Vec<InfoExpr>,
    returns: bool,
    function: &mut Function,
    block: &mut usize,
    module: &mut Module,
    store: Option<usize>,
    declarations: &HashMap<String, Declaration>,
    locals: &mut HashMap<String, Declaration>,
    next_var: &mut usize,
    tail: bool,
) -> Result<(), IRErrorInfo> {
    let mut i = 0;
    let len = statements.len();
    for statement in statements {
        if i != len - 1 || !returns {
            to_ir(
                function,
                block,
                module,
                statement,
                None,
                declarations,
                locals,
                next_var,
                false,
            )?;
        } else {
            to_ir(
                function,
                block,
                module,
                statement,
                store,
                declarations,
                locals,
                next_var,
                tail,
            )?;
        }
        i += 1;
    }

    if (len == 0 || !returns) && store.is_some() {
        function.ir[*block].statements.push(Statement {
            store,
            operation: Operation::LoadLiteral(Value::new(
                EmptyTuple {},
                TypeExpr::Tuple(Vec::new()),
            )),
        });
    }
    Ok(())
}
