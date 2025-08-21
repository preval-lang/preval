use crate::tokeniser::{InfoToken, Keyword, Literal, Operator, Token};

#[derive(Debug)]
pub enum Expr {
    Index(Box<InfoExpr>, Box<InfoExpr>),
    Var(String),
    Literal(Literal),
    Call(Box<InfoExpr>, Vec<InfoExpr>),
    Return(Option<Box<InfoExpr>>),
    Block(Vec<InfoExpr>, bool),
    Let(String, Box<InfoExpr>),
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
    MalformedLet,
    DuplicateName,
}

pub fn parse_expression(tokens: &[InfoToken]) -> Result<InfoExpr, InfoParseError> {
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
                token: Token::Operator(Operator::Assign),
                idx: _,
            }) = tokens.get(2)
            {
                return Ok(InfoExpr {
                    expr: Expr::Let(name.clone(), Box::new(parse_expression(&tokens[3..])?)),
                    idx: *let_idx,
                });
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

    let mut highest_precedence: Option<(i32, usize)> = None;

    for (i, token) in tokens.iter().enumerate() {
        if let Token::Operator(op) = &token.token {
            if let Some(hp) = highest_precedence {
                if op.precidence() > hp.0 {
                    highest_precedence = Some((op.precidence(), i));
                }
            } else {
                highest_precedence = Some((op.precidence(), i));
            }
        }
    }

    if let Some(hp) = highest_precedence {
        match &tokens[hp.1].token {
            Token::Operator(op) => match op {
                Operator::Assign => {
                    todo!("Mutability");
                }
                Operator::Dot => {
                    let left = parse_expression(&tokens[0..hp.1]).unwrap();

                    if let Token::Name(name) = &tokens[hp.1 + 1].token {
                        Ok(InfoExpr {
                            idx: left.idx,
                            expr: Expr::Index(
                                Box::new(left),
                                Box::new(InfoExpr {
                                    idx: tokens[hp.1 + 1].idx,
                                    expr: Expr::Literal(Literal::String(name.clone())),
                                }),
                            ),
                        })
                    } else {
                        Err(InfoParseError {
                            idx: tokens[hp.1 + 1].idx,
                            error: ParseError::ExpectedName,
                        })
                    }
                }
                Operator::Call(args) => {
                    let left = parse_expression(&tokens[0..hp.1]).unwrap();

                    Ok(InfoExpr {
                        idx: left.idx,
                        expr: Expr::Call(
                            Box::new(left),
                            args.iter()
                                .filter(|a| !a.is_empty())
                                .map(|a| parse_expression(&a).unwrap())
                                .collect(),
                        ),
                    })
                }
            },

            _ => unreachable!("Lowest precidence operator index doesn't point to an operator"),
        }
    } else {
        match &tokens[..] {
            [
                InfoToken {
                    token: Token::Literal(literal),
                    idx,
                },
            ] => Ok(InfoExpr {
                idx: *idx,
                expr: Expr::Literal(literal.clone()),
            }),
            [
                InfoToken {
                    token: Token::Name(name),
                    idx,
                },
            ] => Ok(InfoExpr {
                idx: *idx,
                expr: Expr::Var(name.clone()),
            }),
            [
                InfoToken {
                    token: Token::Parens(tokens),
                    idx: _,
                },
            ] => parse_expression(tokens),
            [
                InfoToken {
                    token: Token::Keyword(Keyword::Return),
                    idx,
                },
                rest @ ..,
            ] => Ok(InfoExpr {
                idx: *idx,
                expr: Expr::Return(Some(Box::new(parse_expression(rest)?))),
            }),
            [
                InfoToken {
                    token: Token::Block(tokens),
                    idx,
                },
            ] => Ok(InfoExpr {
                idx: *idx,
                expr: Expr::Block(
                    {
                        let mut out = Vec::new();

                        for token in tokens
                            .split(|tk| tk.token == Token::Semicolon)
                            .filter(|t| !t.is_empty())
                            .map(parse_expression)
                        {
                            out.push(token?);
                        }

                        out
                    },
                    !tokens.last().is_some_and(|tk| tk.token == Token::Semicolon),
                ),
            }),
            [] => Err(InfoParseError {
                idx: 0,
                error: ParseError::ExpectedExpression(Vec::new()),
            }),
            a => Err(InfoParseError {
                idx: a[0].idx,
                error: ParseError::ExpectedExpression(a.to_vec()),
            }),
        }
    }
}
