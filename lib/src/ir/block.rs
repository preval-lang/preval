use crate::ir::IRContext;
use crate::ir::Operation;
use crate::ir::Statement;
use crate::ir::error::IRErrorInfo;
use crate::ir::to_ir;
use crate::parser::expression::InfoExpr;
use crate::typ::type_id;
use crate::value::Value;
use crate::value::primitive::EmptyTuple;

pub fn compile_block<'a>(
	statements: Vec<InfoExpr<'a>>,
	returns: bool,
	block: &mut usize,
	store: Option<usize>,
	tail: bool,
	context: &mut IRContext<'_, 'a>,
) -> Result<(), IRErrorInfo<'a>> {
	let mut i = 0;
	let len = statements.len();
	for statement in statements {
		if i != len - 1 || !returns {
			to_ir(block, statement, None, false, context)?;
		} else {
			to_ir(block, statement, store, tail, context)?;
		}
		i += 1;
	}

	if (len == 0 || !returns) && store.is_some() {
		context.blocks[*block].statements.push(Statement {
			store,
			operation: Operation::LoadLiteral(Value::new(EmptyTuple {}, type_id::empty_tuple)),
		});
	}
	Ok(())
}
