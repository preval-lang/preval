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
	let expr = if let Some(expr) = try_parse_union(tokens, generics)? {
		expr
	} else if let Some(expr) = try_parse_subtype(tokens, generics)? {
		expr
	} else if let Some(expr) = try_parse_generics(tokens, generics)? {
		expr
	} else if let Some(expr) = try_parse_name(tokens, generics)? {
		expr
	} else {
		return Err(InfoParseError {
			span: tokens[0].span.clone(),
			error: ParseError::ExpectedExpression(tokens.to_vec()),
		});
	};

	Ok(expr)
}

fn try_parse_name<'a>(
	tokens: &[InfoToken<'a>],
	generics: &[String],
) -> Result<Option<InfoTypeExpr<'a>>, InfoParseError<'a>> {
	let (name, span) = if let Some(InfoToken {
		token: Token::Name(name),
		span,
	}) = tokens.get(0)
	{
		(name, span)
	} else {
		return Ok(None);
	};
	if let Some(generic) = generics
		.iter()
		.enumerate()
		.find_map(|i| if i.1 == name { Some(i.0) } else { None })
	{
		return Ok(Some(InfoTypeExpr {
			expr: TypeExpr::Parameter(generic),
			idx: span.clone(),
		}));
	} else {
		return Ok(Some(InfoTypeExpr {
			expr: TypeExpr::Name(name.clone(), Vec::new()),
			idx: span.clone(),
		}));
	}
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

	let name = match parse_type(&tokens[..open_idx], generics)? {
		InfoTypeExpr {
			expr: TypeExpr::Name(name, _),
			idx: _,
		} => name,
		_ => {
			return Err(InfoParseError {
				span: tokens[0].span.clone(),
				error: ParseError::ExpectedName,
			});
		}
	};

	for generic_param_tokens in generics_tokens {
		if generic_param_tokens.len() == 1 {
			if let Token::Name(n) = &generic_param_tokens[0].token {
				if n == "_" {
					param_exprs.push(None);
					continue;
				}
			}
		}
		param_exprs.push(Some(parse_type(&generic_param_tokens, generics)?));
	}

	Ok(Some(InfoTypeExpr {
		expr: TypeExpr::Name(name, param_exprs),
		idx: tokens[open_idx].span.clone(),
	}))
}

fn try_parse_subtype<'a>(
	tokens: &[InfoToken<'a>],
	generics: &[String],
) -> Result<Option<InfoTypeExpr<'a>>, InfoParseError<'a>> {
	let (left, right, span) = if let Some((
		idx,
		InfoToken {
			token: Token::DoubleColon,
			span,
		},
	)) = tokens
		.iter()
		.enumerate()
		.rfind(|t| t.1.token == Token::DoubleColon)
	{
		(&tokens[0..idx], &tokens[idx + 1..], span)
	} else {
		return Ok(None);
	};

	if left.len() == 0 {
		panic!(":: with no preceding type {tokens:?}");
	}

	let left = parse_type(left, generics)?;
	let right = parse_type(right, generics)?;

	let (right_name, right_generics) = if let InfoTypeExpr {
		expr: TypeExpr::Name(name, generics),
		idx: _,
	} = right
	{
		(name, generics)
	} else {
		return Err(InfoParseError {
			span: right.idx,
			error: ParseError::ExpectedName,
		});
	};

	Ok(Some(InfoTypeExpr {
		expr: TypeExpr::Subtype(Some(Box::new(left)), right_name, right_generics),
		idx: span.clone(),
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
