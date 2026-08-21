use crate::error::Span;
use crate::parser::expression::ParseError;
use crate::{
	parser::{
		expression::InfoParseError,
		typ::{InfoTypeExpr, parse_type},
		utility::read_punctuated,
	},
	tokeniser::{InfoToken, Token},
	typ::TypeExpr,
};

pub fn expect_function_signature<'a>(
	tokens: &[InfoToken<'a>],
	i: &mut usize,
) -> Result<Signature<'a>, InfoParseError<'a>> {
	if let Token::Name(name) = &tokens[*i].token {
		let name_idx = tokens[*i].span.clone();
		*i += 1;

		let mut args = Vec::new();
		let generics_tokens = if let Token::LessThan = &tokens[*i].token {
			*i += 1;
			let start = *i;
			loop {
				if let Some(InfoToken {
					token: Token::GreaterThan,
					span: _,
				}) = tokens.get(*i)
				{
					break;
				}
				*i += 1;
			}
			*i += 1;
			Some(&tokens[start..*i - 1])
		} else {
			None
		};
		let generics = if let Some(generics_tokens) = generics_tokens {
			let generics = read_punctuated(generics_tokens, Token::Comma)?;
			generics
				.iter()
				.map(|param_tokens| {
					if let [
						InfoToken {
							token: Token::Name(name),
							span: _,
						},
					] = &param_tokens[..]
					{
						name.clone()
					} else {
						panic!("Non name tokens in generic {param_tokens:?}")
					}
				})
				.collect()
		} else {
			Vec::new()
		};

		if let Token::Parens(contents) = &tokens[*i].token {
			for arg_colon_type in read_punctuated(contents, Token::Comma)? {
				if let [
					InfoToken {
						token: Token::Name(name),
						span: _name_idx,
					},
					InfoToken {
						token: Token::Colon,
						span: _colon_idx,
					},
					typ @ ..,
				] = &arg_colon_type[..]
				{
					let typ = parse_type(typ, &generics)?;
					args.push((name.clone(), typ));
				}
			}
			*i += 1;
		} else {
			panic!("Missing function parameters, got {:?}", tokens[*i]);
		}
		let returns = if let Token::Colon = &tokens[*i].token {
			*i += 1;
			let start = *i;
			loop {
				if let Some(InfoToken {
					token: Token::Braces(_) | Token::Semicolon,
					span: _,
				}) = tokens.iter().nth(*i)
				{
					break;
				}
				*i += 1;
			}

			parse_type(&tokens[start..*i], &generics)?
		} else {
			InfoTypeExpr {
				expr: TypeExpr::Tuple(Vec::new()),
				idx: tokens[*i].span.clone(),
			}
		};
		Ok(Signature {
			name: name.clone(),
			name_idx,
			generics,
			args: args.iter().map(|arg| arg.0.clone()).collect(),
			arg_types: args.iter().map(|arg| arg.1.clone()).collect(),
			return_type: returns,
		})
	} else {
		Err(InfoParseError {
			span: tokens[*i].span.clone(),
			error: ParseError::ExpectedFunctionSignature(tokens[*i].clone()),
		})
	}
}

#[derive(Clone, Debug)]
pub struct Signature<'a> {
	pub name: String,
	pub name_idx: Span<'a>,
	pub generics: Vec<String>,
	pub args: Vec<String>,
	pub arg_types: Vec<InfoTypeExpr<'a>>,
	pub return_type: InfoTypeExpr<'a>,
}
