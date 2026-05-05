use crate::value::runtime_type::TypeDeserializer;

#[derive(Debug)]
pub struct IRErrorInfo {
    pub idx: usize,
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
