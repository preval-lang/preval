use crate::typ::{Type, TypeReference};

#[derive(Debug, Clone, PartialEq)]
pub enum TypeError {
    UnknownVariable(String),
    UnknownField(String),
    UnknownType(String),
    IncompatibleTypes {
        expected: TypeReference,
        got: TypeReference,
    },
    NotAStruct(Type),
    NotAFunction(TypeReference),
    IncorrectArgumentCount {
        expected: usize,
        got: usize,
    },
    IncorrectFieldCount {
        expected: usize,
        got: usize,
    },
    DuplicateName(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct InfoTypeError {
    pub idx: usize,
    pub error: TypeError,
}
