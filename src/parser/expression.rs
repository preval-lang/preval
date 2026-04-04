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
    ExpectedTopLevel,
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

// pub fn parse_expression_old(tokens: &[InfoToken]) -> Result<InfoExpr, InfoParseError> {
//     let mut lowest_precendence: Option<(i32, usize)> = None;

//     for (i, token) in tokens.iter().enumerate() {
//         if let Token::Operator(op) = &token.token {
//             if let Some(hp) = lowest_precendence {
//                 if op.precidence() < hp.0 {
//                     lowest_precendence = Some((op.precidence(), i));
//                 }
//             } else {
//                 lowest_precendence = Some((op.precidence(), i));
//             }
//         }
//     }

//     if let Some(lp) = lowest_precendence {
//         match &tokens[lp.1].token {
//             Token::Operator(op) => match op {
//                 Operator::Assign => {
//                     todo!("Mutability");
//                 }
//                 Operator::Dot => {
//                     let left = parse_expression(&tokens[0..lp.1]).unwrap();
//                     if let Token::Name(name) = &tokens[lp.1 + 1].token {
//                         Ok(InfoExpr {
//                             idx: left.idx,
//                             expr: Expr::Index(
//                                 Box::new(left),
//                                 Box::new(InfoExpr {
//                                     idx: tokens[lp.1 + 1].idx,
//                                     expr: Expr::Literal(Value::new(name.to_string())),
//                                 }),
//                             ),
//                         })
//                     } else {
//                         Err(InfoParseError {
//                             idx: tokens[lp.1 + 1].idx,
//                             error: ParseError::ExpectedName,
//                         })
//                     }
//                 }
//                 Operator::Call(args) => {
//                     let left = parse_expression(&tokens[0..lp.1]).unwrap();
//                     Ok(InfoExpr {
//                         idx: left.idx,
//                         expr: Expr::Call(
//                             Box::new(left),
//                             args.iter()
//                                 .filter(|a| !a.is_empty())
//                                 .map(|a| parse_expression(&a).unwrap())
//                                 .collect(),
//                         ),
//                     })
//                 }
//             },

//             _ => unreachable!("Lowest precidence operator index doesn't point to an operator"),
//         }
//     } else {
//         match &tokens[..] {
//             [
//                 InfoToken {
//                     token: Token::Literal(literal),
//                     idx,
//                 },
//             ] => Ok(InfoExpr {
//                 idx: *idx,
//                 expr: Expr::Literal(literal.clone()),
//             }),
//             [
//                 InfoToken {
//                     token: Token::Keyword(Keyword::Bool(value)),
//                     idx,
//                 },
//             ] => Ok(InfoExpr {
//                 idx: *idx,
//                 expr: Expr::Literal(Value::new(*value)),
//             }),
//             [
//                 InfoToken {
//                     token: Token::Name(name),
//                     idx,
//                 },
//             ] => Ok(InfoExpr {
//                 idx: *idx,
//                 expr: Expr::Var(name.clone()),
//             }),
//             [
//                 InfoToken {
//                     token: Token::Parens(tokens),
//                     idx: _,
//                 },
//             ] => parse_expression(tokens),
//             [
//                 InfoToken {
//                     token: Token::Keyword(Keyword::Return),
//                     idx,
//                 },
//                 rest @ ..,
//             ] => Ok(InfoExpr {
//                 idx: *idx,
//                 expr: Expr::Return(Some(Box::new(parse_expression(rest)?))),
//             }),
//             [
//                 InfoToken {
//                     token: Token::Braces(tokens),
//                     idx,
//                 },
//             ] => Ok(InfoExpr {
//                 idx: *idx,
//                 expr: Expr::Block(
//                     {
//                         let mut out = Vec::new();

//                         for token in tokens
//                             .split(|tk| tk.token == Token::Semicolon)
//                             .filter(|t| !t.is_empty())
//                             .map(parse_expression)
//                         {
//                             out.push(token?);
//                         }

//                         out
//                     },
//                     !tokens.last().is_some_and(|tk| tk.token == Token::Semicolon),
//                 ),
//             }),
//             [
//                 InfoToken {
//                     token: Token::Keyword(Keyword::If),
//                     idx,
//                 },
//                 tail @ ..,
//             ] => {
//                 let (cond, then_block, else_keyword, else_block) = match tail {
//                     [
//                         cond @ ..,
//                         InfoToken {
//                             token: Token::Braces(then_contents),
//                             idx: then_idx,
//                         },
//                         InfoToken {
//                             token: Token::Keyword(Keyword::Else),
//                             idx: else_keyword_idx,
//                         },
//                         InfoToken {
//                             token: Token::Braces(else_contents),
//                             idx: else_idx,
//                         },
//                     ] => (
//                         cond,
//                         InfoToken {
//                             token: Token::Braces(then_contents.to_vec()),
//                             idx: *then_idx,
//                         },
//                         Some(InfoToken {
//                             token: Token::Keyword(Keyword::Else),
//                             idx: *else_keyword_idx,
//                         }),
//                         Some(InfoToken {
//                             token: Token::Braces(else_contents.to_vec()),
//                             idx: *else_idx,
//                         }),
//                     ),
//                     _ => todo!("More if forms"),
//                 };

//                 Ok(InfoExpr {
//                     idx: *idx,
//                     expr: Expr::If {
//                         cond: Box::new(parse_expression(cond)?),
//                         then: Box::new(parse_expression(&[then_block])?),
//                         els: if let Some(else_block) = else_block {
//                             Some(Box::new(parse_expression(&[else_block])?))
//                         } else {
//                             None
//                         },
//                     },
//                 })
//             }
//             [] => Err(InfoParseError {
//                 idx: 0,
//                 error: ParseError::ExpectedExpression(Vec::new()),
//             }),
//             a => Err(InfoParseError {
//                 idx: a[0].idx,
//                 error: ParseError::ExpectedExpression(a.to_vec()),
//             }),
//         }
//     }
// }

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

    if let Some(expr) = try_parse_index(tokens)? {
        return Ok(expr);
    }

    if let Some(expr) = try_parse_call(tokens)? {
        return Ok(expr);
    }

    if let Some(expr) = try_parse_dot(tokens)? {
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
            idx: name_idx,
        },
    ] = tokens
    {
        return Ok(Some(InfoExpr {
            expr: Expr::Index(
                Box::new(parse_expression(left)?),
                Box::new(InfoExpr {
                    expr: Expr::Literal(Value::new(name.clone())),
                    idx: *name_idx,
                }),
            ),
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
