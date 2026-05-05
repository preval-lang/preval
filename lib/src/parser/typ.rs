use crate::{
    parser::expression::{InfoParseError, ParseError},
    tokeniser::{InfoToken, Token},
    typ::TypeExpr,
};

#[derive(Debug)]
pub struct InfoTypeExpr {
    pub expr: TypeExpr,
    pub idx: usize,
}

pub fn parse_type(tokens: &[InfoToken]) -> Result<InfoTypeExpr, InfoParseError> {
    if let Some(expr) = try_parse_union(tokens)? {
        return Ok(expr);
    }

    if let Some(expr) = try_parse_name(tokens)? {
        return Ok(expr);
    }

    Err(InfoParseError {
        idx: tokens[0].idx,
        error: ParseError::ExpectedExpression(tokens.to_vec()),
    })
}

fn try_parse_name(tokens: &[InfoToken]) -> Result<Option<InfoTypeExpr>, InfoParseError> {
    if tokens.len() != 1 {
        return Ok(None);
    }
    if let InfoToken {
        idx,
        token: Token::Name(name),
    } = &tokens[0]
    {
        Ok(Some(InfoTypeExpr {
            expr: TypeExpr::Name(name.clone()),
            idx: *idx,
        }))
    } else {
        Ok(None)
    }
}

fn try_parse_union(tokens: &[InfoToken]) -> Result<Option<InfoTypeExpr>, InfoParseError> {
    let union_idx = if let Some(union_idx) = tokens.iter().position(|t| t.token == Token::Union) {
        union_idx
    } else {
        return Ok(None);
    };
    let left = &tokens[..union_idx];
    let right = &tokens[union_idx + 1..];

    let left_expr = parse_type(left)?;
    let right_expr = parse_type(right)?;

    Ok(Some(InfoTypeExpr {
        expr: TypeExpr::Union(Box::new(left_expr.expr), Box::new(right_expr.expr)),
        idx: tokens[union_idx].idx,
    }))
}
