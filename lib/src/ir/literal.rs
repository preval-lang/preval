use crate::ir::Function;
use crate::ir::Operation;
use crate::ir::Statement;
use crate::ir::error::IRErrorInfo;
use crate::tokeniser::Literal;
use crate::typ::Type;
use crate::value::Value;

pub fn literal(
    lit: Literal,
    function: &mut Function,
    block: &mut usize,
    store: Option<usize>,
) -> Result<(), IRErrorInfo> {
    if let Some(store) = store {
        function.ir[*block].statements.push(Statement {
            store: Some(store),
            operation: Operation::LoadLiteral(lit.to_value()),
        });
    }
    Ok(())
}
