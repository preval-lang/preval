use crate::{
	parser::{
		expression::{InfoParseError, ParseError},
		typ::{InfoTypeExpr, parse_type},
		utility::read_punctuated,
	},
	tokeniser::{InfoToken, Token},
};

pub struct Use<'a> {
	pub name: String,
	pub typ: InfoTypeExpr<'a>,
}

pub fn expect_use<'a>(
	tokens: &[InfoToken<'a>],
	i: &mut usize,
) -> Result<Use<'a>, InfoParseError<'a>> {
	*i += 1;
	let start = *i;
	while tokens[*i].token != Token::Semicolon {
		*i += 1;
	}
	let end = *i;
	let items = read_punctuated(&tokens[start..end], Token::DoubleColon)?;
	*i += 1;

	let mut path = Vec::new();

	for item in items {
		if let [
			InfoToken {
				token: Token::Name(name),
				span: _,
			},
		] = &item[..]
		{
			path.push(name.clone());
		} else {
			return Err(InfoParseError {
				error: ParseError::ExpectedName,
				span: item[0].span.clone(),
			}
			.into());
		}
	}

	Ok(Use {
		name: path.last().unwrap().clone(),
		typ: parse_type(&tokens[start..end], &vec![])?,
	})
}
