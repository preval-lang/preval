use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::{
	ir::error::{IRError, IRErrorInfo},
	parser::expression::{InfoParseError, ParseError},
	typ::{InfoTypeError, TypeError},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span<'a> {
	pub file: Cow<'a, str>,
	pub index: usize,
}

#[derive(Debug)]
pub struct InfoError<'a> {
	pub span: Span<'a>,
	pub data: Error<'a>,
}

#[derive(Debug)]
pub enum Error<'a> {
	ParseError(ParseError<'a>),
	TypeError(TypeError),
	IRError(IRError),
}

impl<'a> From<InfoParseError<'a>> for InfoError<'a> {
	fn from(value: InfoParseError<'a>) -> Self {
		Self {
			data: Error::ParseError(value.error),
			span: value.span,
		}
	}
}

impl<'a> From<InfoTypeError<'a>> for InfoError<'a> {
	fn from(value: InfoTypeError<'a>) -> Self {
		Self {
			data: Error::TypeError(value.error),
			span: value.span,
		}
	}
}

impl<'a> From<IRErrorInfo<'a>> for InfoError<'a> {
	fn from(value: IRErrorInfo<'a>) -> Self {
		Self {
			data: Error::IRError(value.error),
			span: value.idx,
		}
	}
}
