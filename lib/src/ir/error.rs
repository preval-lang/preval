use crate::{error::Span, value::runtime_type::TypeDeserializer};

#[derive(Debug)]
pub struct IRErrorInfo<'a> {
	pub idx: Span<'a>,
	pub error: IRError,
}

#[derive(Debug)]
pub enum IRError {
	SymbolUndefined(String),
	SymbolNotCallable(String),
	SymbolNotIndexable(String),
	ExpressionNotCallable(TypeDeserializer),
	TypeMismatch {
		got: TypeDeserializer,
		expected: TypeDeserializer,
	},
	ExtraArgument(),
	NotStorable(String),
	MissingElseBlock(),
}
