use crate::ir::Block;
use crate::ir::Operation;
use crate::ir::Statement;
use crate::ir::error::IRErrorInfo;
use crate::tokeniser::Literal;
use crate::typ::type_id;
use crate::value::Value;

pub fn literal(
    lit: Literal,
    function: &mut Vec<Block>,
    block: &mut usize,
    store: Option<usize>,
) -> Result<(), IRErrorInfo> {
    if let Some(store) = store {
        function[*block].statements.push(Statement {
            store: Some(store),
            operation: Operation::LoadLiteral(match lit {
                // TODO: Add method on Literal for this
                Literal::Bool(b) => Value::new(b, type_id::bool),
                Literal::String(b) => Value::new(b, type_id::String),
                Literal::Usize(b) => Value::new(b, type_id::usize),
            }),
        });
    }
    Ok(())
}
