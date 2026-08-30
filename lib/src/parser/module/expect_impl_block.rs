use std::collections::HashMap;

use crate::{
	expect_token,
	parser::{
		expression::{InfoParseError, ParseError},
		module::{
			expect_body::expect_body, expect_function_signature::expect_function_signature,
			expect_generics::expect_generics, expect_name_and_generics::expect_name_and_generics,
		},
		typ::{InfoTypeExpr, parse_type},
	},
	tokeniser::{InfoToken, Keyword, Token},
	typ::{GenericImplementation, Template, TypeExpr},
};
#[derive(Debug, Clone)]
pub struct ImplBlock<'a> {
	pub members: HashMap<String, Template<'a>>,
	pub generics: Vec<String>,
	pub implementor: InfoTypeExpr<'a>,
	pub implemented: Option<InfoTypeExpr<'a>>,
}

pub fn expect_impl_block<'a>(
	tokens: &[InfoToken<'a>],
	i: &mut usize,
) -> Result<ImplBlock<'a>, InfoParseError<'a>> {
	*i += 1;

	let generics = expect_generics(tokens, i)?;

	let first_type_start = *i;
	while match tokens[*i].token {
		Token::Braces(_) | Token::Keyword(Keyword::For) => false,
		_ => true,
	} {
		*i += 1;
	}
	let first_type_end = *i;
	let (implementor, implemented) = if tokens[*i].token == Token::Keyword(Keyword::For) {
		*i += 1;
		let second_type_start = *i;
		while match tokens[*i].token {
			Token::Braces(_) => false,
			_ => true,
		} {
			*i += 1;
		}
		let second_type_end = *i;
		(
			parse_type(&tokens[second_type_start..second_type_end], &generics)?,
			Some(parse_type(
				&tokens[first_type_start..first_type_end],
				&generics,
			)?),
		)
	} else {
		(
			parse_type(&tokens[first_type_start..first_type_end], &generics)?,
			None,
		)
	};

	let tokens = if let Some(InfoToken {
		token: Token::Braces(contents),
		span: _,
	}) = tokens.get(*i)
	{
		contents
	} else {
		return Err(InfoParseError {
			error: ParseError::ExpectedExpression(vec![tokens[*i].clone()]),
			span: tokens[*i].span.clone(),
		}
		.into());
	};
	*i += 1;

	let mut members: HashMap<String, Template<'a>> = HashMap::new();

	let mut i = 0;
	while i < tokens.len() {
		match tokens[i].token {
			Token::Keyword(Keyword::Fn) => {
				i += 1;
				let sig = expect_function_signature(tokens, &mut i)?;
				let body = expect_body(tokens, &mut i, &sig.generics)?;

				if let Some(_) = members.insert(
					sig.name.clone(),
					Template {
						expr: InfoTypeExpr {
							expr: TypeExpr::Function(
								sig.arg_types,
								Box::new(sig.return_type),
								GenericImplementation::Normal(Box::new(body)),
								sig.args,
							),
							idx: sig.name_idx,
						},
						parameters: sig.generics.len(),
					},
				) {
					panic!("Method {:?}", sig.name)
				}
			}
			Token::Keyword(Keyword::Type) => {
				i += 1;
				let (name, generics) = expect_name_and_generics(tokens, &mut i)?;
				if Token::Assignment != tokens[i].token {
					panic!("expected semicolon");
				}
				i += 1;

				let type_start = i;
				while tokens[i].token != Token::Semicolon {
					i += 1;
				}
				let type_end = i;
				members.insert(
					name,
					Template {
						parameters: generics.len(),
						expr: parse_type(&tokens[type_start..type_end], &generics)?,
					},
				);
				expect_token!(tokens, i, Token::Semicolon);
			}
			ref tk => panic!("expected fn got {tk:?}"),
		}
	}

	Ok(ImplBlock {
		generics,
		members,
		implementor,
		implemented,
	})
}
