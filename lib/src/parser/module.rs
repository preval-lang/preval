use std::{collections::HashMap, usize};

use crate::{
    error::{Error, InfoError},
    ir::{Block, Declaration, Terminal, to_ir},
    parser::{
        expression::{InfoExpr, InfoParseError, ParseError, parse_expression},
        typ::{InfoTypeExpr, parse_type},
        utility::read_punctuated,
    },
    passes::type_check_expr::infer_expr_type,
    tokeniser::{InfoToken, Keyword, Literal, Token},
    typ::{Implementation, Program, Type, TypeError, TypeExpr},
    value::native::NativeFunction,
};

use crate::passes::type_check_expr::Scope;

pub fn parse_module(tokens: &[InfoToken]) -> Result<Program, InfoError> {
    let mut instantiator = Program::new();

    module_pass(tokens, &mut instantiator, true)?;
    module_pass(tokens, &mut instantiator, false)?;

    Ok(instantiator)
}

fn module_pass(
    tokens: &[InfoToken],
    mut instantiator: &mut Program,
    first: bool,
) -> Result<(), InfoError> {
    let mut i = 0;

    while i < tokens.len() {
        match tokens[i].token.clone() {
            Token::Keyword(Keyword::Fn) => {
                i += 1;
                let ((name, name_idx), generics, args, args_types, return_type) =
                    expect_function_signature(tokens, &mut i)?;

                let body = expect_block_or_expr(tokens, &mut i, &generics)?;

                if !first {
                    let mut last_var = args.len();

                    let mut ir = vec![Block {
                        terminal: Terminal::Return(last_var),
                        statements: Vec::new(),
                    }];

                    let mut scope = Scope::new();

                    let mut locals = HashMap::new();
                    for (idx, arg) in args.iter().enumerate() {
                        locals.insert(arg.clone(), Declaration::Variable(idx));
                        scope.insert(
                            arg.clone(),
                            instantiator.instantiate(&args_types[idx], &vec![])?,
                        );
                    }

                    to_ir(
                        &mut ir,
                        &mut 0,
                        body.clone(),
                        Some(last_var),
                        &mut locals,
                        &mut last_var,
                        true,
                    )?;

                    instantiator.add_template(
                        name,
                        InfoTypeExpr {
                            expr: TypeExpr::Function(
                                args_types,
                                Box::new(return_type.clone()),
                                Some(Implementation::Normal(ir)),
                            ),
                            idx: name_idx,
                        },
                    );

                    let generics = (0..generics.len())
                        .map(|i| instantiator.add(Type::Placeholder(i)))
                        .collect::<Vec<_>>();

                    let return_type_ins = instantiator.instantiate(&return_type, &generics)?;

                    let body_type = infer_expr_type(
                        &body,
                        &mut instantiator,
                        &mut scope,
                        return_type_ins,
                        &generics,
                    )?;

                    if !instantiator
                        .compatible(body_type, return_type_ins, 0)
                        .unwrap()
                    {
                        return Err(InfoError {
                            info: return_type.idx,
                            data: Error::TypeError(TypeError::IncompatibleTypes {
                                expected: instantiator.get_type(return_type_ins).unwrap().clone(),
                                got: instantiator.get_type(body_type).unwrap().clone(),
                            }),
                        });
                    }
                } else {
                    instantiator.add_template(
                        name,
                        InfoTypeExpr {
                            expr: TypeExpr::Function(
                                args_types,
                                Box::new(return_type.clone()),
                                None,
                            ),
                            idx: name_idx,
                        },
                    );
                }
            }
            Token::Keyword(Keyword::Struct) => {
                let idx = i;
                i += 1;
                let name = if let Token::Name(name) = &tokens[i].token {
                    Ok(name)
                } else {
                    Err(InfoParseError {
                        idx: tokens[i].idx,
                        error: ParseError::ExpectedName,
                    })
                }?;
                i += 1;
                let generics_tokens = if let Token::LessThan = &tokens[i].token {
                    i += 1;
                    let start = i;
                    loop {
                        if let Some(InfoToken {
                            token: Token::GreaterThan,
                            idx: _,
                        }) = tokens.iter().nth(i)
                        {
                            break;
                        }
                        i += 1;
                    }
                    i += 1;
                    Some(&tokens[start..i - 1])
                } else {
                    None
                };
                let generics = if let Some(generics_tokens) = generics_tokens {
                    let generics = read_punctuated(generics_tokens, Token::Comma)?;
                    generics
                        .iter()
                        .map(|param_tokens| {
                            if let [
                                InfoToken {
                                    token: Token::Name(name),
                                    idx: _,
                                },
                            ] = &param_tokens[..]
                            {
                                name.clone()
                            } else {
                                panic!("Non name tokens in generic {param_tokens:?}")
                            }
                        })
                        .collect()
                } else {
                    Vec::new()
                };
                let block = if let Token::Braces(block) = &tokens[i].token {
                    Ok(block)
                } else {
                    Err(InfoParseError {
                        idx: tokens[i].idx,
                        error: ParseError::ExpectedExpression(tokens[i..].to_vec()),
                    })
                }?;

                let mut fields = HashMap::new();

                for field_colon_type in read_punctuated(block, Token::Comma)? {
                    if let [
                        InfoToken {
                            token: Token::Name(name),
                            idx: _name_idx,
                        },
                        InfoToken {
                            token: Token::Colon,
                            idx: _colon_idx,
                        },
                        typ @ ..,
                    ] = field_colon_type.as_slice()
                    {
                        fields.insert(name.clone(), parse_type(typ, &generics)?);
                    }
                }
                i += 1;

                instantiator.add_template(
                    name.clone(),
                    InfoTypeExpr {
                        expr: TypeExpr::Struct(fields),
                        idx,
                    },
                );
            }
            Token::Keyword(Keyword::Dylib) => {
                i += 1;
                let lib_name = if let InfoToken {
                    idx: _,
                    token: Token::Literal(Literal::String(s)),
                } = &tokens[i]
                {
                    s.clone()
                } else {
                    return Err(InfoParseError {
                        idx: tokens[i].idx,
                        error: ParseError::ExpectedString(tokens[i].clone()),
                    }
                    .into());
                };

                i += 1;
                i += 1;

                let ((name, name_idx), generics, _, args, return_type) =
                    expect_function_signature(tokens, &mut i)?;

                let generics = (0..generics.len())
                    .map(|i| instantiator.add(Type::Placeholder(i)))
                    .collect::<Vec<_>>();

                instantiator.instantiate(&return_type, &generics)?;
                for arg in &args {
                    instantiator.instantiate(arg, &generics)?;
                }

                if tokens[i].token != Token::Semicolon {
                    return Err(InfoParseError {
                        idx: tokens[i].idx,
                        error: ParseError::ExpectedSemicolon(tokens[i].clone()),
                    }
                    .into());
                }
                i += 1;

                instantiator.add_template(
                    name.clone(),
                    InfoTypeExpr {
                        expr: TypeExpr::Function(
                            args,
                            Box::new(return_type),
                            if first {
                                None
                            } else {
                                Some(Implementation::Native(NativeFunction {
                                    lib_name,
                                    func_name: name,
                                }))
                            },
                        ),
                        idx: name_idx,
                    },
                );
            }
            _tk => {
                return Err(InfoParseError {
                    idx: tokens[i].idx,
                    error: ParseError::ExpectedTopLevel,
                }
                .into());
            }
        }
    }
    Ok(())
}

pub fn expect_function_signature(
    tokens: &[InfoToken],
    i: &mut usize,
) -> Result<
    (
        (String, usize),
        Vec<String>,
        Vec<String>,
        Vec<InfoTypeExpr>,
        InfoTypeExpr,
    ),
    InfoParseError,
> {
    if let Token::Name(name) = &tokens[*i].token {
        let name_idx = tokens[*i].idx;
        *i += 1;

        let mut args = Vec::new();
        let generics_tokens = if let Token::LessThan = &tokens[*i].token {
            *i += 1;
            let start = *i;
            loop {
                if let Some(InfoToken {
                    token: Token::GreaterThan,
                    idx: _,
                }) = tokens.get(*i)
                {
                    break;
                }
                *i += 1;
            }
            *i += 1;
            Some(&tokens[start..*i - 1])
        } else {
            None
        };
        let generics = if let Some(generics_tokens) = generics_tokens {
            let generics = read_punctuated(generics_tokens, Token::Comma)?;
            generics
                .iter()
                .map(|param_tokens| {
                    if let [
                        InfoToken {
                            token: Token::Name(name),
                            idx: _,
                        },
                    ] = &param_tokens[..]
                    {
                        name.clone()
                    } else {
                        panic!("Non name tokens in generic {param_tokens:?}")
                    }
                })
                .collect()
        } else {
            Vec::new()
        };

        if let Token::Parens(contents) = &tokens[*i].token {
            for arg_colon_type in read_punctuated(contents, Token::Comma)? {
                if let [
                    InfoToken {
                        token: Token::Name(name),
                        idx: _name_idx,
                    },
                    InfoToken {
                        token: Token::Colon,
                        idx: _colon_idx,
                    },
                    typ @ ..,
                ] = &arg_colon_type[..]
                {
                    let typ = parse_type(typ, &generics)?;
                    args.push((name.clone(), typ));
                }
            }
            *i += 1;
        } else {
            panic!("Missing function parameters, got {:?}", tokens[*i]);
        }
        let returns = if let Token::Colon = &tokens[*i].token {
            *i += 1;
            let start = *i;
            loop {
                if let Some(InfoToken {
                    token: Token::Braces(_) | Token::Semicolon,
                    idx: _,
                }) = tokens.iter().nth(*i)
                {
                    break;
                }
                *i += 1;
            }

            parse_type(&tokens[start..*i], &generics)?
        } else {
            InfoTypeExpr {
                expr: TypeExpr::Tuple(Vec::new()),
                idx: tokens[*i].idx,
            }
        };
        Ok((
            (name.clone(), name_idx),
            generics,
            args.iter().map(|arg| arg.0.clone()).collect(),
            args.iter().map(|arg| arg.1.clone()).collect(),
            returns,
        ))
    } else {
        Err(InfoParseError {
            idx: *i,
            error: ParseError::ExpectedFunctionSignature(tokens[*i].clone()),
        })
    }
}

pub fn expect_block_or_expr(
    tokens: &[InfoToken],
    i: &mut usize,
    generics: &[String],
) -> Result<InfoExpr, InfoParseError> {
    if let Some(InfoToken {
        token: Token::Braces(_),
        idx: _,
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
