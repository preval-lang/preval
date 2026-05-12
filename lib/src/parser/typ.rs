use serde::{Deserialize, Serialize};

use crate::{
    parser::{
        expression::{InfoParseError, ParseError},
        utility::read_punctuated,
    },
    tokeniser::{InfoToken, Token},
    typ::TypeExpr,
};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Hash)]
pub struct InfoTypeExpr {
    pub expr: TypeExpr,
    pub idx: usize,
}

pub fn parse_type(
    tokens: &[InfoToken],
    generics: &[String],
) -> Result<InfoTypeExpr, InfoParseError> {
    if let Some(expr) = try_parse_union(tokens, generics)? {
        return Ok(expr);
    }

    if let Some(expr) = try_parse_name(tokens, generics)? {
        return Ok(expr);
    }

    if let Some(expr) = try_parse_generics(tokens, generics)? {
        return Ok(expr);
    }

    Err(InfoParseError {
        idx: tokens[0].idx,
        error: ParseError::ExpectedExpression(tokens.to_vec()),
    })
}

fn try_parse_name(
    tokens: &[InfoToken],
    generics: &[String],
) -> Result<Option<InfoTypeExpr>, InfoParseError> {
    if tokens.len() != 1 {
        return Ok(None);
    }
    if let InfoToken {
        idx,
        token: Token::Name(name),
    } = &tokens[0]
    {
        if let Some(generic) = generics
            .iter()
            .enumerate()
            .find_map(|i| if i.1 == name { Some(i.0) } else { None })
        {
            return Ok(Some(InfoTypeExpr {
                expr: TypeExpr::Parameter(generic),
                idx: *idx,
            }));
        }
        Ok(Some(InfoTypeExpr {
            expr: TypeExpr::Name(name.clone()),
            idx: *idx,
        }))
    } else {
        Ok(None)
    }
}

fn try_parse_generics(
    tokens: &[InfoToken],
    generics: &[String],
) -> Result<Option<InfoTypeExpr>, InfoParseError> {
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
                    idx: tokens[i].idx,
                    error: todo!("parse error for unclosed generics"),
                });
            }
        }
        i += 1;
    }

    let contents = &tokens[open_idx + 1..i];

    let generics_tokens = read_punctuated(contents, Token::Comma)?;

    let mut param_exprs = Vec::new();

    let base = parse_type(&tokens[open_idx - 1..], generics)?;

    for generic_param_tokens in generics_tokens {
        param_exprs.push(parse_type(&generic_param_tokens, generics)?)
    }

    Ok(Some(InfoTypeExpr {
        expr: TypeExpr::Generics(Box::new(base), param_exprs),
        idx: open_idx,
    }))
}

fn try_parse_union(
    tokens: &[InfoToken],
    generics: &[String],
) -> Result<Option<InfoTypeExpr>, InfoParseError> {
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
        idx: tokens[union_idx].idx,
    }))
}
