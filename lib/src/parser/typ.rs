use crate::{
	error::Span,
	parser::{
		expression::{InfoParseError, ParseError},
		utility::read_punctuated,
	},
	tokeniser::{InfoToken, Token},
	typ::TypeExpr,
};

#[derive(Debug, Clone)]
pub struct InfoTypeExpr<'a> {
	pub expr: TypeExpr<'a>,
	pub idx: Span<'a>,
}

pub fn parse_type<'a>(
	tokens: &[InfoToken<'a>],
	generics: &[String],
) -> Result<InfoTypeExpr<'a>, InfoParseError<'a>> {
	if let Some(expr) = try_parse_union(tokens, generics)? {
		return Ok(expr);
	}

	if let Some(expr) = try_parse_generics(tokens, generics)? {
		return Ok(expr);
	}

	if let Some(expr) = try_parse_name(tokens, generics)? {
		return Ok(expr);
	}

	Err(InfoParseError {
		span: tokens[0].span.clone(),
		error: ParseError::ExpectedExpression(tokens.to_vec()),
	})
}

fn try_parse_name<'a>(
	tokens: &[InfoToken<'a>],
	generics: &[String],
) -> Result<Option<InfoTypeExpr<'a>>, InfoParseError<'a>> {
	let global = if tokens[0].token == Token::DoubleColon {
		true
	} else {
		false
	};

	let parts = read_punctuated(&tokens[if global { 1 } else { 0 }..], Token::DoubleColon)?;

	let mut span = tokens[0].span.clone();

	if parts.len() == 1 {
		let name = match &parts[0][0].token {
			Token::Name(name) => name,
			_ => return Ok(None),
		};
		if let Some(generic) = generics
			.iter()
			.enumerate()
			.find_map(|i| if i.1 == name { Some(i.0) } else { None })
		{
			return Ok(Some(InfoTypeExpr {
				expr: TypeExpr::Parameter(generic),
				idx: span,
			}));
		}
	}

	let mut strings = Vec::new();

	for part in parts {
		if part.len() != 1 {
			return Err(InfoParseError {
				span,
				error: ParseError::ExpectedName,
			});
		}
		span = part[0].span.clone();
		match &part[0].token {
			Token::Name(name) => strings.push(name.clone()),
			_ => {
				return Err(InfoParseError {
					span,
					error: ParseError::ExpectedName,
				});
			}
		}
	}

	Ok(Some(InfoTypeExpr {
		expr: TypeExpr::Name(strings, global),
		idx: span,
	}))
}

fn try_parse_generics<'a>(
	tokens: &[InfoToken<'a>],
	generics: &[String],
) -> Result<Option<InfoTypeExpr<'a>>, InfoParseError<'a>> {
	let open_idx = if let Some(open_idx) = tokens.iter().position(|t| t.token == Token::LessThan) {
		open_idx
	} else {
		return Ok(None);
	};

	let mut inside = 0;
	let mut i = open_idx;
	loop {
		if let Token::LessThan = tokens[i].token {
			inside += 1;
		} else if let Token::GreaterThan = tokens[i].token {
			inside -= 1;
			if inside == 0 {
				break;
			}
			if inside < 0 {
				return Err(InfoParseError {
					span: tokens[i].span.clone(),
					error: ParseError::UnclosedAngleBrackets,
				});
			}
		}
		i += 1;
	}

	let contents = &tokens[open_idx + 1..i];

	let generics_tokens = read_punctuated(contents, Token::Comma)?;

	let mut param_exprs = Vec::new();

	let base = parse_type(&tokens[..open_idx], generics)?;

	for generic_param_tokens in generics_tokens {
		param_exprs.push(parse_type(&generic_param_tokens, generics)?)
	}

	Ok(Some(InfoTypeExpr {
		expr: TypeExpr::Generics(Box::new(base), param_exprs),
		idx: tokens[open_idx].span.clone(),
	}))
}

fn try_parse_union<'a>(
	tokens: &[InfoToken<'a>],
	generics: &[String],
) -> Result<Option<InfoTypeExpr<'a>>, InfoParseError<'a>> {
	let union_idx = if let Some(union_idx) = tokens.iter().position(|t| t.token == Token::Union) {
		union_idx
	} else {
		return Ok(None);
	};
	let left = &tokens[..union_idx];
	let right = &tokens[union_idx + 1..];

	let left_expr = parse_type(left, generics)?;
	let right_expr = parse_type(right, generics)?;

	Ok(Some(InfoTypeExpr {
		expr: TypeExpr::Union(Box::new(left_expr), Box::new(right_expr)),
		idx: tokens[union_idx].span.clone(),
	}))
}
