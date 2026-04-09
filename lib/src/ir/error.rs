use crate::value::typ::Type;

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
    ExpressionNotCallable(Type),
    TypeMismatch { got: Type, expected: Type },
    ExtraArgument(),
    NotStorable(String),
    MissingElseBlock(),
}
