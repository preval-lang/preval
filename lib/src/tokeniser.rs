use std::{borrow::Cow, fmt::Debug};

use crate::{
	error::Span,
	typ::{ConcreteType, IntegerSize},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
	Let,
	Return,
	Fn,
	If,
	Else,
	Bool(bool),
	Struct,
	Dylib,
	Guard,
	Is,
	Use,
	Mod,
}

impl TryFrom<&str> for Keyword {
	type Error = ();

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		match value {
			"let" => Ok(Keyword::Let),
			"return" => Ok(Keyword::Return),
			"fn" => Ok(Keyword::Fn),
			"if" => Ok(Keyword::If),
			"else" => Ok(Keyword::Else),
			"true" => Ok(Keyword::Bool(true)),
			"false" => Ok(Keyword::Bool(false)),
			"struct" => Ok(Keyword::Struct),
			"dylib" => Ok(Keyword::Dylib),
			"guard" => Ok(Keyword::Guard),
			"is" => Ok(Keyword::Is),
			"use" => Ok(Keyword::Use),
			"mod" => Ok(Keyword::Mod),
			_ => Err(()),
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token<'a> {
	Name(String),
	Keyword(Keyword),
	Literal(Literal),
	Parens(Vec<InfoToken<'a>>),
	Braces(Vec<InfoToken<'a>>),
	Index(Vec<InfoToken<'a>>),
	Semicolon,
	Colon,
	Comma,
	Dot,
	Assignment,
	Union,
	LessThan,
	GreaterThan,
	DoubleColon,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
	Bool(bool),
	String(String),
	Usize(usize),
}

impl Literal {
	pub fn get_type(&self) -> ConcreteType {
		match self {
			Literal::Bool(_) => ConcreteType::Bool,
			Literal::String(_) => ConcreteType::String,
			Literal::Usize(_) => ConcreteType::Integer {
				size: IntegerSize::Size,
				signed: false,
			},
		}
	}
}

#[derive(Clone)]
pub struct InfoToken<'a> {
	pub token: Token<'a>,
	pub span: Span<'a>,
}

impl PartialEq for InfoToken<'_> {
	fn eq(&self, other: &Self) -> bool {
		self.token == other.token
	}
}

impl Debug for InfoToken<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		Token::fmt(&self.token, f)
	}
}

#[derive(Debug)]
pub struct TokeniseErrorInfo<'a> {
	pub idx: Span<'a>,
	pub error: TokeniseError,
}

#[derive(Debug)]
pub enum TokeniseError {
	UnclosedParens,
	UnclosedQuotes,
	ExpectedToken(char),
	ExpectedNumber(String),
}

#[derive(Debug)]
pub struct EOF {}

pub fn get_line_and_column(input: &str, idx: usize) -> Result<(usize, usize), EOF> {
	let mut col = 1;
	let mut line = 1;

	for (i, c) in input.char_indices() {
		if i == idx {
			return Ok((line, col));
		}
		if c == '\n' {
			line += 1;
			col = 1;
		} else {
			col += 1;
		}
	}

	Err(EOF {})
}

pub fn tokenise<'a>(
	input: &str,
	offset: usize,
	file: Cow<'a, str>,
) -> Result<Vec<InfoToken<'a>>, TokeniseErrorInfo<'a>> {
	let mut out = Vec::new();

	let mut i = 0;

	loop {
		match input.chars().nth(i) {
			None => break,
			Some(c) if c.is_alphabetic() || c == '_' => {
				out.push(read_name(input, &mut i, offset, file.clone()));
			}
			Some('.') => {
				out.push(InfoToken {
					token: Token::Dot,
					span: Span {
						index: offset + i,
						file: file.clone(),
					},
				});
				i += 1;
			}
			Some('=') => {
				out.push(InfoToken {
					token: Token::Assignment,
					span: Span {
						index: offset + i,
						file: file.clone(),
					},
				});
				i += 1;
			}
			Some(';') => {
				out.push(InfoToken {
					token: Token::Semicolon,
					span: Span {
						index: offset + i,
						file: file.clone(),
					},
				});
				i += 1;
			}
			Some(':') => out.push(double_token(
				input,
				&mut i,
				offset,
				file.clone(),
				Token::Colon,
				Token::DoubleColon,
			)?),
			Some(',') => {
				out.push(InfoToken {
					token: Token::Comma,
					span: Span {
						index: offset + i,
						file: file.clone(),
					},
				});
				i += 1;
			}
			Some('|') => {
				out.push(InfoToken {
					token: Token::Union,
					span: Span {
						index: offset + i,
						file: file.clone(),
					},
				});
				i += 1;
			}
			Some('<') => {
				out.push(InfoToken {
					token: Token::LessThan,
					span: Span {
						index: offset + i,
						file: file.clone(),
					},
				});
				i += 1;
			}
			Some('>') => {
				out.push(InfoToken {
					token: Token::GreaterThan,
					span: Span {
						index: offset + i,
						file: file.clone(),
					},
				});
				i += 1;
			}
			Some('(') => {
				let (idx, contents) = read_brackets(input, &mut i, offset, '(', ')', file.clone())?;
				out.push(InfoToken {
					token: Token::Parens(contents),
					span: idx,
				});
			}
			Some('{') => {
				let (idx, contents) = read_brackets(input, &mut i, offset, '{', '}', file.clone())?;
				out.push(InfoToken {
					token: Token::Braces(contents),
					span: idx,
				});
			}
			Some('[') => {
				let (idx, contents) = read_brackets(input, &mut i, offset, '[', ']', file.clone())?;
				out.push(InfoToken {
					token: Token::Index(contents),
					span: idx,
				});
			}
			Some('"') => {
				out.push(read_string(input, &mut i, offset, file.clone())?);
			}
			Some(c) if c.is_numeric() => {
				out.push(read_number(input, &mut i, offset, file.clone())?);
			}
			Some(c) if c.is_whitespace() => i += 1,
			Some(a) => {
				return Err(TokeniseErrorInfo {
					idx: Span {
						index: offset + i,
						file: file.clone(),
					},
					error: TokeniseError::ExpectedToken(a),
				});
			}
		}
	}

	Ok(out)
}

fn double_token<'a>(
	input: &str,
	i: &mut usize,
	offset: usize,
	file: Cow<'a, str>,
	single: Token<'a>,
	double: Token<'a>,
) -> Result<InfoToken<'a>, TokeniseErrorInfo<'a>> {
	let first = input.chars().nth(*i);
	*i += 1;
	Ok(InfoToken {
		token: if first == input.chars().nth(*i) {
			*i += 1;
			double
		} else {
			single
		},
		span: Span {
			index: offset + *i,
			file: file.clone(),
		},
	})
}

fn read_number<'a>(
	input: &str,
	i: &mut usize,
	offset: usize,
	file: Cow<'a, str>,
) -> Result<InfoToken<'a>, TokeniseErrorInfo<'a>> {
	let start = *i;

	let mut number = String::new();

	loop {
		let c = input.chars().nth(*i);
		if c.is_none() || !(c.unwrap().is_numeric() || c.unwrap() == '_') {
			return Ok(InfoToken {
				span: Span {
					index: offset + start,
					file: file.clone(),
				},
				token: if let Ok(num) = number.parse::<usize>() {
					Token::Literal(Literal::Usize(num))
				} else {
					return Err(TokeniseErrorInfo {
						idx: Span {
							index: offset + start,
							file,
						},
						error: TokeniseError::ExpectedNumber(number),
					});
				},
			});
		}
		number.push(c.unwrap());
		*i += 1;
	}
}

fn read_name<'a>(input: &str, i: &mut usize, offset: usize, file: Cow<'a, str>) -> InfoToken<'a> {
	let start = *i;

	let mut name = String::new();

	loop {
		let c = input.chars().nth(*i);
		if c.is_none() || !(c.unwrap().is_alphanumeric() || c.unwrap() == '_') {
			return InfoToken {
				span: Span {
					index: offset + start,
					file,
				},
				token: if let Ok(keyword) = Keyword::try_from(name.as_str()) {
					Token::Keyword(keyword)
				} else {
					Token::Name(name)
				},
			};
		}
		name.push(c.unwrap());
		*i += 1;
	}
}

fn read_brackets<'a>(
	input: &str,
	i: &mut usize,
	offset: usize,
	open: char,
	close: char,
	file: Cow<'a, str>,
) -> Result<(Span<'a>, Vec<InfoToken<'a>>), TokeniseErrorInfo<'a>> {
	let start = *i;

	let mut contents = String::new();

	let mut open_parens = 0;

	loop {
		let c = input.chars().nth(*i);
		match c {
			Some(c) if c == open => {
				open_parens += 1;
				if open_parens != 1 {
					contents.push(open);
				}
			}
			Some(c) if c == close => {
				open_parens -= 1;
				if open_parens == 0 {
					*i += 1;
					return Ok((
						Span {
							index: offset + start,
							file: file.clone(),
						},
						(tokenise(&contents, offset + start + 1, file)?),
					));
				} else {
					contents.push(close);
				}
			}
			Some(c) => {
				contents.push(c);
			}
			None => {
				return Err(TokeniseErrorInfo {
					idx: Span {
						index: offset + start,
						file,
					},
					error: TokeniseError::UnclosedParens,
				});
			}
		}
		*i += 1;
	}
}

fn read_string<'a>(
	input: &str,
	i: &mut usize,
	offset: usize,
	file: Cow<'a, str>,
) -> Result<InfoToken<'a>, TokeniseErrorInfo<'a>> {
	// TODO: escape sequences

	let start = *i;

	let mut contents = String::new();

	*i += 1;

	loop {
		let c = input.chars().nth(*i);
		match c {
			Some('"') => {
				*i += 1;
				return Ok(InfoToken {
					span: Span {
						index: offset + start,
						file,
					},
					token: Token::Literal(Literal::String(contents)),
				});
			}
			Some(c) => {
				contents.push(c);
			}
			None => {
				return Err(TokeniseErrorInfo {
					idx: Span {
						file,
						index: offset + start,
					},
					error: TokeniseError::UnclosedQuotes,
				});
			}
		}
		*i += 1;
	}
}
