use crate::{
    parser::expression::{InfoParseError, ParseError},
    tokeniser::{InfoToken, Token},
    typ::{Name, TypeExpr},
};

pub fn parse_type(tokens: &[InfoToken]) -> Result<TypeExpr, InfoParseError> {
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

fn try_parse_name(tokens: &[InfoToken]) -> Result<Option<TypeExpr>, InfoParseError> {
    if tokens.len() != 1 {
        return Ok(None);
    }
    if let InfoToken {
        idx,
        token: Token::Name(name),
    } = &tokens[0]
    {
        Ok(Some(TypeExpr::Named(Name {
            path: vec![name.clone()],
            generics: Vec::new(),
        })))
    } else {
        Ok(None)
    }
}

fn try_parse_union(tokens: &[InfoToken]) -> Result<Option<TypeExpr>, InfoParseError> {
    let union_idx = if let Some(union_idx) = tokens.iter().position(|t| t.token == Token::Union) {
        union_idx
    } else {
        return Ok(None);
    };
    let left = &tokens[..union_idx];
    let right = &tokens[union_idx + 1..];

    let left_expr = parse_type(left)?;
    let right_expr = parse_type(right)?;

    Ok(Some(TypeExpr::Union(
        Box::new(left_expr),
        Box::new(right_expr),
    )))
}
