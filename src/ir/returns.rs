use crate::ir::Declaration;
use crate::ir::Function;
use crate::ir::Module;
use crate::ir::Terminal;
use crate::ir::error::IRErrorInfo;
use crate::ir::to_ir;
use crate::parser::expression::InfoExpr;
use std::collections::HashMap;

pub fn returns(
    value_expr: Option<Box<InfoExpr>>,
    function: &mut Function,
    block: &mut usize,
    module: &mut Module,
    store: Option<usize>,
    declarations: &HashMap<String, Declaration>,
    locals: &mut HashMap<String, Declaration>,
    next_var: &mut usize,
) -> Result<(), IRErrorInfo> {
    function.ir[*block].terminal = Terminal::Return(if let Some(value_expr) = value_expr {
        let return_var = {
            *next_var += 1;
            *next_var
        };
        to_ir(
            function,
            block,
            module,
            *value_expr,
            Some(return_var),
            declarations,
            locals,
            next_var,
        )?;
        Some(return_var)
    } else {
        None
    });
    Ok(())
}
