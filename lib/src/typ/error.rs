use crate::{error::Span, typ::Type};

#[derive(Debug, Clone, PartialEq)]
pub enum TypeError {
	UnknownVariable(String),
	UnknownField(String),
	UnknownType(Vec<String>),
	IncompatibleTypes { expected: Type, got: Type },
	NotAStruct(Type),
	NotAFunction(Type),
	IncorrectArgumentCount { expected: usize, got: usize },
	IncorrectFieldCount { expected: usize, got: usize },
	DuplicateName(String),
}

#[derive(Debug, Clone)]
pub struct InfoTypeError<'a> {
	pub span: Span<'a>,
	pub error: TypeError,
}
