use std::collections::HashMap;

use crate::parser::typ::{InfoTypeExpr, parse_type};
use crate::parser::utility::read_punctuated;
use crate::tokeniser::Literal;
use crate::typ::{Instantiator, TypeExpr};
use crate::{
    ir::error::{IRError, IRErrorInfo},
    tokeniser::{InfoToken, Keyword, Token},
};

#[derive(Debug, Clone)]
pub enum Expr {
    Index(Box<InfoExpr>, Box<InfoExpr>),
    Name(InfoTypeExpr),
    Literal(Literal),
    Call(Box<InfoExpr>, Vec<InfoExpr>),
    Return(Option<Box<InfoExpr>>),
    Block(Vec<InfoExpr>, bool),
    Let(String, Box<InfoExpr>),
    If {
        cond: Box<InfoExpr>,
        then: Box<InfoExpr>,
        els: Option<Box<InfoExpr>>,
    },
    InitializeStruct(InfoTypeExpr, HashMap<String, InfoExpr>),
    Access(Box<InfoExpr>, String),
    Guard {
        dependency: Box<InfoExpr>,
        body: Box<InfoExpr>,
    },
    Is {
        name: String,
        typ: InfoTypeExpr,
    },
}

#[derive(Debug, Clone)]
pub struct InfoExpr {
    pub idx: usize,
    pub expr: Expr,
}

#[derive(Debug)]
pub struct InfoParseError {
    pub idx: usize,
    pub error: ParseError,
}

#[derive(Debug)]
pub enum ParseError {
    ExpectedName,
    ExpectedExpression(Vec<InfoToken>),
    ExpectedString(InfoToken),
    ExpectedTopLevel,
    ExpectedFunctionSignature(InfoToken),
    ExpectedSemicolon(InfoToken),
    ExpectedAssign,
    DuplicateName,
    TypeUndefined(Vec<InfoToken>),
    IRError(IRError),
}

impl From<IRErrorInfo> for InfoParseError {
    fn from(value: IRErrorInfo) -> Self {
        InfoParseError {
            idx: value.idx,
            error: ParseError::IRError(value.error),
        }
    }
}

pub fn parse_expression(
    tokens: &[InfoToken],
    ins: &mut Instantiator,
    generics: &[String],
) -> Result<InfoExpr, InfoParseError> {
    if let Some(expr) = try_parse_parens(tokens, ins, generics)? {
        return Ok(expr);
    }

    if let Some(expr) = try_parse_block(tokens, ins, generics)? {
        return Ok(expr);
    }

    if let Some(expr) = try_parse_let(tokens, ins, generics)? {
        return Ok(expr);
    }

    if let Some(expr) = try_parse_return(tokens, ins, generics)? {
        return Ok(expr);
    }

    if let Some(expr) = try_parse_if(tokens, ins, generics)? {
        return Ok(expr);
    }

    if let Some(expr) = try_parse_guard(tokens, ins, generics)? {
        return Ok(expr);
    }

    if let Some(expr) = try_parse_index(tokens, ins, generics)? {
        return Ok(expr);
    }

    if let Some(expr) = try_parse_call(tokens, ins, generics)? {
        return Ok(expr);
    }

    if let Some(expr) = try_parse_dot(tokens, ins, generics)? {
        return Ok(expr);
    }

    if let Some(expr) = try_parse_is(tokens, ins, generics)? {
        return Ok(expr);
    }

    if let Some(expr) = try_parse_struct(tokens, ins, generics)? {
        return Ok(expr);
    }

    if let Some(expr) = try_parse_literal(tokens)? {
        return Ok(expr);
    }

    if let Some(expr) = try_parse_boolean(tokens)? {
        return Ok(expr);
    }

    if let Some(expr) = try_parse_name(tokens, generics)? {
        return Ok(expr);
    }

    Err(InfoParseError {
        idx: 0,
        error: ParseError::ExpectedExpression(tokens.to_vec()),
    })
}

fn try_parse_let(
    tokens: &[InfoToken],
    ins: &mut Instantiator,
    generics: &[String],
) -> Result<Option<InfoExpr>, InfoParseError> {
    if let Some(InfoToken {
        token: Token::Keyword(Keyword::Let),
        idx: let_idx,
    }) = tokens.get(0)
    {
        if let Some(InfoToken {
            token: Token::Name(name),
            idx: _,
        }) = tokens.get(1)
        {
            if let Some(InfoToken {
                token: Token::Assignment,
                idx: _,
            }) = tokens.get(2)
            {
                return Ok(Some(InfoExpr {
                    expr: Expr::Let(
                        name.clone(),
                        Box::new(parse_expression(&tokens[3..], ins, generics)?),
                    ),
                    idx: *let_idx,
                }));
            } else {
                return Err(InfoParseError {
                    idx: *let_idx,
                    error: ParseError::ExpectedAssign,
                });
            }
        } else {
            return Err(InfoParseError {
                idx: *let_idx,
                error: ParseError::ExpectedName,
            });
        }
    }
    Ok(None)
}

fn try_parse_guard(
    tokens: &[InfoToken],
    ins: &mut Instantiator,
    generics: &[String],
) -> Result<Option<InfoExpr>, InfoParseError> {
    if let [
        InfoToken {
            token: Token::Keyword(Keyword::Guard),
            idx: guard_idx,
        },
        InfoToken {
            token: Token::Parens(dependency),
            idx: _,
        },
        rest @ ..,
    ] = tokens
    {
        return Ok(Some(InfoExpr {
            idx: *guard_idx,
            expr: Expr::Guard {
                dependency: Box::new(parse_expression(dependency, ins, generics)?),
                body: Box::new(parse_expression(rest, ins, generics)?),
            },
        }));
    }
    Ok(None)
}

fn try_parse_is(
    tokens: &[InfoToken],
    ins: &mut Instantiator,
    generics: &[String],
) -> Result<Option<InfoExpr>, InfoParseError> {
    if let [
        InfoToken {
            token: Token::Name(name),
            idx: _,
        },
        InfoToken {
            token: Token::Keyword(Keyword::Is),
            idx: is_idx,
        },
        type_expr @ ..,
    ] = tokens
    {
        return Ok(Some(InfoExpr {
            idx: *is_idx,
            expr: Expr::Is {
                name: name.clone(),
                typ: parse_type(type_expr, generics)?,
            },
        }));
    }
    Ok(None)
}

fn try_parse_if(
    tokens: &[InfoToken],
    ins: &mut Instantiator,
    generics: &[String],
) -> Result<Option<InfoExpr>, InfoParseError> {
    if let [
        InfoToken {
            token: Token::Keyword(Keyword::If),
            idx: if_idx,
        },
        condition @ ..,
        then_block @ InfoToken {
            token: Token::Braces(_),
            idx: _,
        },
        InfoToken {
            token: Token::Keyword(Keyword::Else),
            idx: _,
        },
        else_block @ InfoToken {
            token: Token::Braces(_),
            idx: _,
        },
    ] = tokens
    {
        return Ok(Some(InfoExpr {
            expr: Expr::If {
                cond: Box::new(parse_expression(condition, ins, generics)?),
                then: Box::new(parse_expression(&[then_block.clone()], ins, generics)?),
                els: Some(Box::new(parse_expression(
                    &[else_block.clone()],
                    ins,
                    generics,
                )?)),
            },
            idx: *if_idx,
        }));
    }
    Ok(None)
}

fn try_parse_struct(
    tokens: &[InfoToken],
    ins: &mut Instantiator,
    generics: &[String],
) -> Result<Option<InfoExpr>, InfoParseError> {
    if let [
        type_tokens @ ..,
        InfoToken {
            token: Token::Braces(contents),
            idx: _brace_idx,
        },
    ] = tokens
    {
        let mut fields = HashMap::new();
        for name_colon_value in read_punctuated(contents, Token::Comma)? {
            if let [
                InfoToken {
                    token: Token::Name(name),
                    idx: _name_idx,
                },
                InfoToken {
                    token: Token::Colon,
                    idx: _colon_idx,
                },
                value @ ..,
            ] = &name_colon_value[..]
            {
                let value = parse_expression(value, ins, generics)?;
                fields.insert(name.clone(), value);
            }
        }

        let type_expr = parse_type(type_tokens, &generics)?;

        Ok(Some(InfoExpr {
            expr: Expr::InitializeStruct(type_expr, fields),
            idx: type_tokens[0].idx,
        }))
    } else {
        Ok(None)
    }
}

fn try_parse_return(
    tokens: &[InfoToken],
    ins: &mut Instantiator,
    generics: &[String],
) -> Result<Option<InfoExpr>, InfoParseError> {
    if let [
        InfoToken {
            token: Token::Keyword(Keyword::Return),
            idx,
        },
        return_tokens @ ..,
    ] = tokens
    {
        return Ok(Some(InfoExpr {
            expr: Expr::Return({
                if return_tokens.is_empty() {
                    None
                } else {
                    Some(Box::new(parse_expression(return_tokens, ins, generics)?))
                }
            }),
            idx: *idx,
        }));
    }
    Ok(None)
}

fn try_parse_index(
    tokens: &[InfoToken],
    ins: &mut Instantiator,
    generics: &[String],
) -> Result<Option<InfoExpr>, InfoParseError> {
    if let [
        left @ ..,
        InfoToken {
            idx,
            token: Token::Index(index),
        },
    ] = tokens
    {
        return Ok(Some(InfoExpr {
            expr: Expr::Index(
                Box::new(parse_expression(left, ins, generics)?),
                Box::new(parse_expression(index, ins, generics)?),
            ),
            idx: *idx,
        }));
    }
    Ok(None)
}

fn try_parse_parens(
    tokens: &[InfoToken],
    ins: &mut Instantiator,
    generics: &[String],
) -> Result<Option<InfoExpr>, InfoParseError> {
    if let [
        InfoToken {
            token: Token::Parens(contents),
            idx: _,
        },
    ] = tokens
    {
        return Ok(Some(parse_expression(contents, ins, generics)?));
    }
    Ok(None)
}

fn try_parse_dot(
    tokens: &[InfoToken],
    ins: &mut Instantiator,
    generics: &[String],
) -> Result<Option<InfoExpr>, InfoParseError> {
    if let [
        left @ ..,
        InfoToken {
            token: Token::Dot,
            idx,
        },
        InfoToken {
            token: Token::Name(name),
            idx: _name_idx,
        },
    ] = tokens
    {
        return Ok(Some(InfoExpr {
            expr: Expr::Access(
                Box::new(parse_expression(left, ins, generics)?),
                name.clone(),
            ),
            idx: *idx,
        }));
    }

    Ok(None)
}

fn try_parse_call(
    tokens: &[InfoToken],
    ins: &mut Instantiator,
    generics: &[String],
) -> Result<Option<InfoExpr>, InfoParseError> {
    if let [
        left @ ..,
        InfoToken {
            token: Token::Parens(contents),
            idx,
        },
    ] = tokens
    {
        return Ok(Some(InfoExpr {
            expr: Expr::Call(Box::new(parse_expression(left, ins, generics)?), {
                let mut out = Vec::new();
                for tokens in read_punctuated(contents, Token::Comma)? {
                    out.push(parse_expression(&tokens, ins, generics)?);
                }
                out
            }),
            idx: *idx,
        }));
    }
    Ok(None)
}

fn try_parse_block(
    tokens: &[InfoToken],
    ins: &mut Instantiator,
    generics: &[String],
) -> Result<Option<InfoExpr>, InfoParseError> {
    if let [
        InfoToken {
            token: Token::Braces(contents),
            idx,
        },
    ] = tokens
    {
        let mut out = Vec::new();
        let returns = if let Some(token) = tokens.last() {
            if token.token == Token::Semicolon {
                false
            } else {
                true
            }
        } else {
            false
        };
        if !contents.is_empty() {
            for tokens in read_punctuated(contents, Token::Semicolon)? {
                out.push(parse_expression(&tokens, ins, generics)?);
            }
        }
        return Ok(Some(InfoExpr {
            expr: Expr::Block(out, returns),
            idx: *idx,
        }));
    }
    Ok(None)
}

fn try_parse_name(
    tokens: &[InfoToken],
    generics: &[String],
) -> Result<Option<InfoExpr>, InfoParseError> {
    Ok(Some(InfoExpr {
        idx: tokens[0].idx,
        expr: Expr::Name(parse_type(tokens, generics)?),
    }))
}

fn try_parse_literal(tokens: &[InfoToken]) -> Result<Option<InfoExpr>, InfoParseError> {
    if let [
        InfoToken {
            token: Token::Literal(value),
            idx,
        },
    ] = tokens
    {
        return Ok(Some(InfoExpr {
            expr: Expr::Literal(value.clone()),
            idx: *idx,
        }));
    }
    Ok(None)
}

fn try_parse_boolean(tokens: &[InfoToken]) -> Result<Option<InfoExpr>, InfoParseError> {
    if let [
        InfoToken {
            token: Token::Keyword(Keyword::Bool(value)),
            idx,
        },
    ] = tokens
    {
        return Ok(Some(InfoExpr {
            expr: Expr::Literal(Literal::Bool(*value)),
            idx: *idx,
        }));
    }
    Ok(None)
}
