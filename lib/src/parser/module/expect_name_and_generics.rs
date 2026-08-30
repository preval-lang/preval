use crate::{
	parser::{
		expression::{InfoParseError, ParseError},
		module::expect_generics::expect_generics,
	},
	tokeniser::{InfoToken, Token},
};

pub fn expect_name_and_generics<'a>(
	tokens: &[InfoToken<'a>],
	i: &mut usize,
) -> Result<(String, Vec<String>), InfoParseError<'a>> {
	let name = if let Token::Name(name) = &tokens[*i].token {
		Ok(name)
	} else {
		Err(InfoParseError {
			span: tokens[*i].span.clone(),
			error: ParseError::ExpectedName,
		})
	}?;
	*i += 1;

	let generics = expect_generics(tokens, i)?;

	Ok((name.clone(), generics))
}
