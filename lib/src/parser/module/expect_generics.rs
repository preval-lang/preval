use crate::{
	parser::{expression::InfoParseError, utility::read_punctuated},
	tokeniser::{InfoToken, Token},
};

pub fn expect_generics<'a>(
	tokens: &[InfoToken<'a>],
	i: &mut usize,
) -> Result<Vec<String>, InfoParseError<'a>> {
	let generics_tokens = if let Token::LessThan = &tokens[*i].token {
		*i += 1;
		let start = *i;
		loop {
			if let Some(InfoToken {
				token: Token::GreaterThan,
				span: _,
			}) = tokens.iter().nth(*i)
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
	Ok(if let Some(generics_tokens) = generics_tokens {
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
	})
}
