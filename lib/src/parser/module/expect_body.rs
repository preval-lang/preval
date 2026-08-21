use crate::{
	parser::expression::{InfoExpr, InfoParseError, parse_expression},
	tokeniser::{InfoToken, Token},
};

pub fn expect_body<'a>(
	tokens: &[InfoToken<'a>],
	i: &mut usize,
	generics: &[String],
) -> Result<InfoExpr<'a>, InfoParseError<'a>> {
	if let Some(InfoToken {
		token: Token::Braces(_),
		span: _,
	}) = tokens.get(*i)
	{
		*i += 1;
		return parse_expression(&tokens[*i - 1..*i], generics);
	} else {
		let mut out = Vec::new();
		loop {
			let token = tokens[*i].clone();

			if token.token == Token::Semicolon {
				*i += 1;
				break;
			}

			out.push(token);

			*i += 1;
		}
		return parse_expression(&out, generics);
	}
}
