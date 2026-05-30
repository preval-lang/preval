use std::fmt::Display;

use crate::{
    ir::error::{IRError, IRErrorInfo},
    parser::expression::{InfoParseError, ParseError},
    typ::{InfoTypeError, TypeError},
};

type ErrorInfo = usize;

#[derive(Debug)]
pub struct InfoError {
    pub info: ErrorInfo,
    pub data: Error,
}

#[derive(Debug)]
pub enum Error {
    ParseError(ParseError),
    TypeError(TypeError),
    IRError(IRError),
}

impl From<InfoParseError> for InfoError {
    fn from(value: InfoParseError) -> Self {
        Self {
            data: Error::ParseError(value.error),
            info: value.idx,
        }
    }
}

impl From<InfoTypeError> for InfoError {
    fn from(value: InfoTypeError) -> Self {
        Self {
            data: Error::TypeError(value.error),
            info: value.idx,
        }
    }
}

impl From<IRErrorInfo> for InfoError {
    fn from(value: IRErrorInfo) -> Self {
        Self {
            data: Error::IRError(value.error),
            info: value.idx,
        }
    }
}
