use crate::ir::IRContext;
use crate::ir::{Operation, Statement};
use crate::parser::typ::InfoTypeExpr;
use crate::typ::TypeExpr;

pub fn variable<'a>(
	name: InfoTypeExpr<'a>,
	block: &mut usize,
	store: Option<usize>,
	context: &mut IRContext<'_, 'a>,
) {
	if let Some(store) = store {
		match name.expr {
			TypeExpr::Name(name, generics)
				if generics.len() == 0 && context.locals.contains_key(&name) =>
			{
				match context.locals[&name] {
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
							.instantiate(&name, context.generics)
							.expect("pass errors in types to IRError"),
					),
				});
			}
		}
	}
}
