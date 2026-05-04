use crate::value::runtime_type::RuntimeType;

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
    ExpressionNotCallable(RuntimeType),
    TypeMismatch {
        got: RuntimeType,
        expected: RuntimeType,
    },
    ExtraArgument(),
    NotStorable(String),
    MissingElseBlock(),
}
