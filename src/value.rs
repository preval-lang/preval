use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    String(String),
    Usize(usize),
    Bool(bool),
    Char(char),
    EmptyTuple,
    IO,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Usize(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::EmptyTuple => write!(f, "()"),
            Value::IO => write!(f, "IO"),
            Value::Char(c) => write!(f, "'{c}'"),
        }
    }
}
