use std::collections::HashMap;

use crate::error::Span;
use crate::parser::typ::{InfoTypeExpr, parse_type};
use crate::parser::utility::read_punctuated;
use crate::tokeniser::Literal;
use crate::{
	ir::error::IRError,
	tokeniser::{InfoToken, Keyword, Token},
};

#[derive(Debug, Clone)]
pub enum Expr<'a> {
	Index(Box<InfoExpr<'a>>, Box<InfoExpr<'a>>),
	Name(InfoTypeExpr<'a>),
	Literal(Literal),
	Call(Box<InfoExpr<'a>>, Vec<InfoExpr<'a>>),
	Return(Option<Box<InfoExpr<'a>>>),
	Block(Vec<InfoExpr<'a>>, bool),
	Let(String, Box<InfoExpr<'a>>),
	If {
		cond: Box<InfoExpr<'a>>,
		then: Box<InfoExpr<'a>>,
		els: Option<Box<InfoExpr<'a>>>,
	},
	InitializeStruct(InfoTypeExpr<'a>, HashMap<String, InfoExpr<'a>>),
	Access(Box<InfoExpr<'a>>, String),
	Guard {
		dependency: Box<InfoExpr<'a>>,
		body: Box<InfoExpr<'a>>,
	},
	Is {
		name: String,
		typ: InfoTypeExpr<'a>,
	},
}

#[derive(Debug, Clone)]
pub struct InfoExpr<'a> {
	pub idx: Span<'a>,
	pub expr: Expr<'a>,
}

pub struct InfoParseError<'a> {
	pub span: Span<'a>,
	pub error: ParseError<'a>,
}

#[derive(Debug)]
pub enum ParseError<'a> {
	ExpectedName,
	ExpectedExpression(Vec<InfoToken<'a>>),
	ExpectedString(InfoToken<'a>),
	ExpectedTopLevel,
	ExpectedFunctionSignature(InfoToken<'a>),
	ExpectedSemicolon(InfoToken<'a>),
	ExpectedAssign,
	DuplicateName,
	TypeUndefined(Vec<InfoToken<'a>>),
	IRError(IRError),
	UnclosedAngleBrackets,
}

pub fn parse_expression<'a>(
	tokens: &[InfoToken<'a>],
	generics: &[String],
) -> Result<InfoExpr<'a>, InfoParseError<'a>> {
	if let Some(expr) = try_parse_parens(tokens, generics)? {
		return Ok(expr);
	}

	if let Some(expr) = try_parse_block(tokens, generics)? {
		return Ok(expr);
	}

	if let Some(expr) = try_parse_let(tokens, generics)? {
		return Ok(expr);
	}

	if let Some(expr) = try_parse_return(tokens, generics)? {
		return Ok(expr);
	}

	if let Some(expr) = try_parse_if(tokens, generics)? {
		return Ok(expr);
	}

	if let Some(expr) = try_parse_guard(tokens, generics)? {
		return Ok(expr);
	}

	if let Some(expr) = try_parse_index(tokens, generics)? {
		return Ok(expr);
	}

	if let Some(expr) = try_parse_call(tokens, generics)? {
		return Ok(expr);
	}

	if let Some(expr) = try_parse_dot(tokens, generics)? {
		return Ok(expr);
	}

	if let Some(expr) = try_parse_is(tokens, generics)? {
		return Ok(expr);
	}

	if let Some(expr) = try_parse_struct(tokens, generics)? {
		return Ok(expr);
	}

	if let Some(expr) = try_parse_literal(tokens)? {
		return Ok(expr);
	}

	if let Some(expr) = try_parse_boolean(tokens)? {
		return Ok(expr);
	}

	if let Some(expr) = try_parse_name(tokens, generics)? {
		return Ok(expr);
	}

	todo!("expected expression error span should be passed to parse_expression")
}

fn try_parse_let<'a>(
	tokens: &[InfoToken<'a>],
	generics: &[String],
) -> Result<Option<InfoExpr<'a>>, InfoParseError<'a>> {
	if let Some(InfoToken {
		token: Token::Keyword(Keyword::Let),
		span: let_idx,
	}) = tokens.get(0)
	{
		if let Some(InfoToken {
			token: Token::Name(name),
			span: _,
		}) = tokens.get(1)
		{
			if let Some(InfoToken {
				token: Token::Assignment,
				span: _,
			}) = tokens.get(2)
			{
				return Ok(Some(InfoExpr {
					expr: Expr::Let(
						name.clone(),
						Box::new(parse_expression(&tokens[3..], generics)?),
					),
					idx: let_idx.clone(),
				}));
			} else {
				return Err(InfoParseError {
					span: let_idx.clone(),
					error: ParseError::ExpectedAssign,
				});
			}
		} else {
			return Err(InfoParseError {
				span: let_idx.clone(),
				error: ParseError::ExpectedName,
			});
		}
	}
	Ok(None)
}

fn try_parse_guard<'a>(
	tokens: &[InfoToken<'a>],
	generics: &[String],
) -> Result<Option<InfoExpr<'a>>, InfoParseError<'a>> {
	if let [
		InfoToken {
			token: Token::Keyword(Keyword::Guard),
			span: guard_idx,
		},
		InfoToken {
			token: Token::Parens(dependency),
			span: _,
		},
		rest @ ..,
	] = tokens
	{
		return Ok(Some(InfoExpr {
			idx: guard_idx.clone(),
			expr: Expr::Guard {
				dependency: Box::new(parse_expression(dependency, generics)?),
				body: Box::new(parse_expression(rest, generics)?),
			},
		}));
	}
	Ok(None)
}

fn try_parse_is<'a>(
	tokens: &[InfoToken<'a>],
	generics: &[String],
) -> Result<Option<InfoExpr<'a>>, InfoParseError<'a>> {
	if let [
		InfoToken {
			token: Token::Name(name),
			span: _,
		},
		InfoToken {
			token: Token::Keyword(Keyword::Is),
			span: is_idx,
		},
		type_expr @ ..,
	] = tokens
	{
		return Ok(Some(InfoExpr {
			idx: is_idx.clone(),
			expr: Expr::Is {
				name: name.clone(),
				typ: parse_type(type_expr, generics)?,
			},
		}));
	}
	Ok(None)
}

fn try_parse_if<'a>(
	tokens: &[InfoToken<'a>],
	generics: &[String],
) -> Result<Option<InfoExpr<'a>>, InfoParseError<'a>> {
	if let [
		InfoToken {
			token: Token::Keyword(Keyword::If),
			span: if_idx,
		},
		condition @ ..,
		then_block @ InfoToken {
			token: Token::Braces(_),
			span: _,
		},
		InfoToken {
			token: Token::Keyword(Keyword::Else),
			span: _,
		},
		else_block @ InfoToken {
			token: Token::Braces(_),
			span: _,
		},
	] = tokens
	{
		return Ok(Some(InfoExpr {
			expr: Expr::If {
				cond: Box::new(parse_expression(condition, generics)?),
				then: Box::new(parse_expression(&[then_block.clone()], generics)?),
				els: Some(Box::new(parse_expression(&[else_block.clone()], generics)?)),
			},
			idx: if_idx.clone(),
		}));
	}
	Ok(None)
}

fn try_parse_struct<'a>(
	tokens: &[InfoToken<'a>],
	generics: &[String],
) -> Result<Option<InfoExpr<'a>>, InfoParseError<'a>> {
	if let [
		type_tokens @ ..,
		InfoToken {
			token: Token::Braces(contents),
			span: _brace_idx,
		},
	] = tokens
	{
		let mut fields = HashMap::new();
		for name_colon_value in read_punctuated(contents, Token::Comma)? {
			if let [
				InfoToken {
					token: Token::Name(name),
					span: _name_idx,
				},
				InfoToken {
					token: Token::Colon,
					span: _colon_idx,
				},
				value @ ..,
			] = &name_colon_value[..]
			{
				let value = parse_expression(value, generics)?;
				fields.insert(name.clone(), value);
			}
		}

		let type_expr = parse_type(type_tokens, &generics)?;

		Ok(Some(InfoExpr {
			expr: Expr::InitializeStruct(type_expr, fields),
			idx: type_tokens[0].span.clone(),
		}))
	} else {
		Ok(None)
	}
}

fn try_parse_return<'a>(
	tokens: &[InfoToken<'a>],
	generics: &[String],
) -> Result<Option<InfoExpr<'a>>, InfoParseError<'a>> {
	if let [
		InfoToken {
			token: Token::Keyword(Keyword::Return),
			span: idx,
		},
		return_tokens @ ..,
	] = tokens
	{
		return Ok(Some(InfoExpr {
			expr: Expr::Return({
				if return_tokens.is_empty() {
					None
				} else {
					Some(Box::new(parse_expression(return_tokens, generics)?))
				}
			}),
			idx: idx.clone(),
		}));
	}
	Ok(None)
}

fn try_parse_index<'a>(
	tokens: &[InfoToken<'a>],
	generics: &[String],
) -> Result<Option<InfoExpr<'a>>, InfoParseError<'a>> {
	if let [
		left @ ..,
		InfoToken {
			span: idx,
			token: Token::Index(index),
		},
	] = tokens
	{
		return Ok(Some(InfoExpr {
			expr: Expr::Index(
				Box::new(parse_expression(left, generics)?),
				Box::new(parse_expression(index, generics)?),
			),
			idx: idx.clone(),
		}));
	}
	Ok(None)
}

fn try_parse_parens<'a>(
	tokens: &[InfoToken<'a>],
	generics: &[String],
) -> Result<Option<InfoExpr<'a>>, InfoParseError<'a>> {
	if let [
		InfoToken {
			token: Token::Parens(contents),
			span: _,
		},
	] = tokens
	{
		return Ok(Some(parse_expression(contents, generics)?));
	}
	Ok(None)
}

fn try_parse_dot<'a>(
	tokens: &[InfoToken<'a>],
	generics: &[String],
) -> Result<Option<InfoExpr<'a>>, InfoParseError<'a>> {
	if let [
		left @ ..,
		InfoToken {
			token: Token::Dot,
			span: idx,
		},
		InfoToken {
			token: Token::Name(name),
			span: _name_idx,
		},
	] = tokens
	{
		return Ok(Some(InfoExpr {
			expr: Expr::Access(Box::new(parse_expression(left, generics)?), name.clone()),
			idx: idx.clone(),
		}));
	}

	Ok(None)
}

fn try_parse_call<'a>(
	tokens: &[InfoToken<'a>],
	generics: &[String],
) -> Result<Option<InfoExpr<'a>>, InfoParseError<'a>> {
	if let [
		left @ ..,
		InfoToken {
			token: Token::Parens(contents),
			span: idx,
		},
	] = tokens
	{
		return Ok(Some(InfoExpr {
			expr: Expr::Call(Box::new(parse_expression(left, generics)?), {
				let mut out = Vec::new();
				for tokens in read_punctuated(contents, Token::Comma)? {
					out.push(parse_expression(&tokens, generics)?);
				}
				out
			}),
			idx: idx.clone(),
		}));
	}
	Ok(None)
}

fn try_parse_block<'a>(
	tokens: &[InfoToken<'a>],
	generics: &[String],
) -> Result<Option<InfoExpr<'a>>, InfoParseError<'a>> {
	if let [
		InfoToken {
			token: Token::Braces(contents),
			span: idx,
		},
	] = tokens
	{
		let mut out = Vec::new();
		let returns = if let Some(token) = tokens.last() {
			if token.token == Token::Semicolon {
				false
			} else {
				true
			}
		} else {
			false
		};
		if !contents.is_empty() {
			for tokens in read_punctuated(contents, Token::Semicolon)? {
				out.push(parse_expression(&tokens, generics)?);
			}
		}
		return Ok(Some(InfoExpr {
			expr: Expr::Block(out, returns),
			idx: idx.clone(),
		}));
	}
	Ok(None)
}

fn try_parse_name<'a>(
	tokens: &[InfoToken<'a>],
	generics: &[String],
) -> Result<Option<InfoExpr<'a>>, InfoParseError<'a>> {
	Ok(Some(InfoExpr {
		idx: tokens[0].span.clone(),
		expr: Expr::Name(parse_type(tokens, generics)?),
	}))
}

fn try_parse_literal<'a>(
	tokens: &[InfoToken<'a>],
) -> Result<Option<InfoExpr<'a>>, InfoParseError<'a>> {
	if let [
		InfoToken {
			token: Token::Literal(value),
			span: idx,
		},
	] = tokens
	{
		return Ok(Some(InfoExpr {
			expr: Expr::Literal(value.clone()),
			idx: idx.clone(),
		}));
	}
	Ok(None)
}

fn try_parse_boolean<'a>(
	tokens: &[InfoToken<'a>],
) -> Result<Option<InfoExpr<'a>>, InfoParseError<'a>> {
	if let [
		InfoToken {
			token: Token::Keyword(Keyword::Bool(value)),
			span: idx,
		},
	] = tokens
	{
		return Ok(Some(InfoExpr {
			expr: Expr::Literal(Literal::Bool(*value)),
			idx: idx.clone(),
		}));
	}
	Ok(None)
}
