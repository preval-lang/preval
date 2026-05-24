use std::collections::HashMap;

use crate::ir::error::IRErrorInfo;
use crate::ir::{Block, to_ir};

use crate::parser::typ::InfoTypeExpr;
use crate::{
    ir::{Declaration, Function, Module, Operation, Statement},
    parser::expression::InfoExpr,
};
pub fn generics(
    base: Box<InfoExpr>,
    _: Vec<InfoTypeExpr>,
    function: &mut Vec<Block>,
    block: &mut usize,
    module: &mut Module,
    store: Option<usize>,
    declarations: &HashMap<String, Declaration>,
    locals: &mut HashMap<String, Declaration>,
    next_var: &mut usize,
    tail: bool,
) -> Result<(), IRErrorInfo> {
    todo!("remove this its unused");
}
