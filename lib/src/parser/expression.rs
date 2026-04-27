use std::collections::HashMap;

use crate::parser::utility::read_punctuated;
use crate::value::Value;
use crate::{
    ir::error::{IRError, IRErrorInfo},
    tokeniser::{InfoToken, Keyword, Token},
};

#[derive(Debug)]
pub enum Expr {
    Index(Box<InfoExpr>, Box<InfoExpr>),
    Var(String),
    Literal(Value),
    Call(Box<InfoExpr>, Vec<InfoExpr>),
    Return(Option<Box<InfoExpr>>),
    Block(Vec<InfoExpr>, bool),
    Let(String, Box<InfoExpr>),
    If {
        cond: Box<InfoExpr>,
        then: Box<InfoExpr>,
        els: Option<Box<InfoExpr>>,
    },
    InitializeStruct(String, HashMap<String, InfoExpr>),
    Access(Box<InfoExpr>, String),
    Guard {
        dependency: Box<InfoExpr>,
        body: Box<InfoExpr>,
    },
}

#[derive(Debug)]
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

pub fn parse_expression(tokens: &[InfoToken]) -> Result<InfoExpr, InfoParseError> {
    if let Some(expr) = try_parse_parens(tokens)? {
        return Ok(expr);
    }

    if let Some(expr) = try_parse_block(tokens)? {
        return Ok(expr);
    }

    if let Some(expr) = try_parse_let(tokens)? {
        return Ok(expr);
    }

    if let Some(expr) = try_parse_return(tokens)? {
        return Ok(expr);
    }

    if let Some(expr) = try_parse_if(tokens)? {
        return Ok(expr);
    }

    if let Some(expr) = try_parse_guard(tokens)? {
        return Ok(expr);
    }

    if let Some(expr) = try_parse_index(tokens)? {
        return Ok(expr);
    }

    if let Some(expr) = try_parse_call(tokens)? {
        return Ok(expr);
    }

    if let Some(expr) = try_parse_dot(tokens)? {
        return Ok(expr);
    }

    if let Some(expr) = try_parse_struct(tokens)? {
        return Ok(expr);
    }

    if let Some(expr) = try_parse_name(tokens)? {
        return Ok(expr);
    }

    if let Some(expr) = try_parse_literal(tokens)? {
        return Ok(expr);
    }

    if let Some(expr) = try_parse_boolean(tokens)? {
        return Ok(expr);
    }

    Err(InfoParseError {
        idx: 0,
        error: ParseError::ExpectedExpression(tokens.to_vec()),
    })
}

fn try_parse_let(tokens: &[InfoToken]) -> Result<Option<InfoExpr>, InfoParseError> {
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
                    expr: Expr::Let(name.clone(), Box::new(parse_expression(&tokens[3..])?)),
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

fn try_parse_guard(tokens: &[InfoToken]) -> Result<Option<InfoExpr>, InfoParseError> {
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
                dependency: Box::new(parse_expression(dependency)?),
                body: Box::new(parse_expression(rest)?),
            },
        }));
    }
    Ok(None)
}

fn try_parse_if(tokens: &[InfoToken]) -> Result<Option<InfoExpr>, InfoParseError> {
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
                cond: Box::new(parse_expression(condition)?),
                then: Box::new(parse_expression(&[then_block.clone()])?),
                els: Some(Box::new(parse_expression(&[else_block.clone()])?)),
            },
            idx: *if_idx,
        }));
    }
    Ok(None)
}

fn try_parse_struct(tokens: &[InfoToken]) -> Result<Option<InfoExpr>, InfoParseError> {
    if let [
        InfoToken {
            token: Token::Name(name),
            idx: name_idx,
        },
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
                let value = parse_expression(value)?;
                fields.insert(name.clone(), value);
            }
        }
        Ok(Some(InfoExpr {
            expr: Expr::InitializeStruct(name.clone(), fields),
            idx: *name_idx,
        }))
    } else {
        Ok(None)
    }
}

fn try_parse_return(tokens: &[InfoToken]) -> Result<Option<InfoExpr>, InfoParseError> {
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
                    Some(Box::new(parse_expression(return_tokens)?))
                }
            }),
            idx: *idx,
        }));
    }
    Ok(None)
}

fn try_parse_index(tokens: &[InfoToken]) -> Result<Option<InfoExpr>, InfoParseError> {
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
                Box::new(parse_expression(left)?),
                Box::new(parse_expression(index)?),
            ),
            idx: *idx,
        }));
    }
    Ok(None)
}

fn try_parse_parens(tokens: &[InfoToken]) -> Result<Option<InfoExpr>, InfoParseError> {
    if let [
        InfoToken {
            token: Token::Parens(contents),
            idx: _,
        },
    ] = tokens
    {
        return Ok(Some(parse_expression(contents)?));
    }
    Ok(None)
}

fn try_parse_dot(tokens: &[InfoToken]) -> Result<Option<InfoExpr>, InfoParseError> {
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
            expr: Expr::Access(Box::new(parse_expression(left)?), name.clone()),
            idx: *idx,
        }));
    }

    Ok(None)
}

fn try_parse_call(tokens: &[InfoToken]) -> Result<Option<InfoExpr>, InfoParseError> {
    if let [
        left @ ..,
        InfoToken {
            token: Token::Parens(contents),
            idx,
        },
    ] = tokens
    {
        return Ok(Some(InfoExpr {
            expr: Expr::Call(Box::new(parse_expression(left)?), {
                let mut out = Vec::new();
                for tokens in read_punctuated(contents, Token::Comma)? {
                    out.push(parse_expression(&tokens)?);
                }
                out
            }),
            idx: *idx,
        }));
    }
    Ok(None)
}

fn try_parse_block(tokens: &[InfoToken]) -> Result<Option<InfoExpr>, InfoParseError> {
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
                out.push(parse_expression(&tokens)?);
            }
        }
        return Ok(Some(InfoExpr {
            expr: Expr::Block(out, returns),
            idx: *idx,
        }));
    }
    Ok(None)
}

fn try_parse_name(tokens: &[InfoToken]) -> Result<Option<InfoExpr>, InfoParseError> {
    if let [
        InfoToken {
            token: Token::Name(name),
            idx,
        },
    ] = tokens
    {
        return Ok(Some(InfoExpr {
            expr: Expr::Var(name.clone()),
            idx: *idx,
        }));
    }
    Ok(None)
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
            expr: Expr::Literal(Value::new(*value)),
            idx: *idx,
        }));
    }
    Ok(None)
}
