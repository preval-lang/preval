use crate::ir::IRContext;
use crate::ir::error::IRErrorInfo;
use crate::ir::{Operation, Statement};
use crate::parser::typ::InfoTypeExpr;
use crate::typ::TypeExpr;

pub fn variable<'a>(
	name: InfoTypeExpr<'a>,
	block: &mut usize,
	store: Option<usize>,
	context: &mut IRContext<'_, 'a>,
) -> Result<(), IRErrorInfo<'a>> {
	if let Some(store) = store {
		match name.expr {
			TypeExpr::Name(name, global)
				if !global && name.len() == 1 && context.locals.contains_key(&name[0]) =>
			{
				match context.locals[&name[0]] {
					v => {
						context.blocks[*block].statements.push(Statement {
							store: Some(store),
							operation: Operation::LoadLocal { src: v },
						});
					}
				}
			}
			_ => {
				context.blocks[*block].statements.push(Statement {
					store: Some(store),
					operation: Operation::LoadFunction(
						context
							.ins
							.instantiate(&name, context.generics, context.prefix)
							.expect("pass errors in types to IRError"),
					),
				});
			}
		}
	}
	Ok(())
}
