use crate::typ::Type;

#[derive(Debug, Clone, PartialEq)]
pub enum TypeError {
    UnknownVariable(String),
    UnknownField(String),
    UnknownType(String),
    IncompatibleTypes { expected: Type, got: Type },
    NotAStruct(Type),
    NotAFunction(Type),
    IncorrectArgumentCount { expected: usize, got: usize },
    IncorrectFieldCount { expected: usize, got: usize },
    DuplicateName(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct InfoTypeError {
    pub idx: usize,
    pub error: TypeError,
}
