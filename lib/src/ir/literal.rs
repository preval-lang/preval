use crate::ir::Function;
use crate::ir::Operation;
use crate::ir::Statement;
use crate::ir::error::IRErrorInfo;
use crate::value::Value;

pub fn literal(
    lit: Value,
    function: &mut Function,
    block: &mut usize,
    store: Option<usize>,
) -> Result<(), IRErrorInfo> {
    if let Some(store) = store {
        function.ir[*block].statements.push(Statement::Operation(
            Operation::LoadLiteral(lit),
            Some(store),
        ));
    }
    Ok(())
}
