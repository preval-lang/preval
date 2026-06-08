use crate::parser::expression::InfoParseError;
use crate::tokeniser::{InfoToken, Token};

pub fn read_punctuated<'a>(
	tokens: &[InfoToken<'a>],
	separator: Token,
) -> Result<Vec<Vec<InfoToken<'a>>>, InfoParseError<'a>> {
	let mut result = Vec::new();
	let mut current = Vec::new();
	for token in tokens {
		if token.token == separator {
			if !current.is_empty() {
				result.push(current);
				current = Vec::new();
			}
		} else {
			current.push(token.clone());
		}
	}
	if !current.is_empty() {
		result.push(current);
	}
	Ok(result)
}
