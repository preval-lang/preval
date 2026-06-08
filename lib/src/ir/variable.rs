use std::collections::HashMap;

use crate::ir::Block;
use crate::ir::error::IRErrorInfo;

use crate::ir::{Declaration, Operation, Statement};
use crate::parser::typ::InfoTypeExpr;
use crate::typ::TypeExpr;

pub fn variable<'a>(
	name: InfoTypeExpr<'a>,
	function: &mut Vec<Block>,
	block: &mut usize,
	store: Option<usize>,
	locals: &mut HashMap<String, Declaration>,
) -> Result<(), IRErrorInfo<'a>> {
	if let Some(store) = store {
		match name.expr {
			TypeExpr::Name(name) if name.len() == 1 && locals.contains_key(&name[0]) => {
				match locals[&name[0]] {
					Declaration::Variable(v) => {
						function[*block].statements.push(Statement {
							store: Some(store),
							operation: Operation::LoadLocal { src: v },
						});
					}
				}
			}
			_ => {
				function[*block].statements.push(Statement {
					store: Some(store),
					operation: Operation::LoadFunction(name.into()),
				});
			}
		}
	}
	Ok(())
}
