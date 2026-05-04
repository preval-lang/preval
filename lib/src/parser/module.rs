use std::{collections::HashMap, usize};

use crate::{
    ir::{Block, Declaration, Function, Module, Terminal, to_ir},
    parser::{
        expression::{InfoExpr, InfoParseError, ParseError, parse_expression},
        typ::parse_type,
        utility::read_punctuated,
    },
    passes::type_check_expr::infer_expr_type,
    tokeniser::{InfoToken, Keyword, Literal, Token},
    typ::{ConcreteType, InfoTypeError, Instantiator, Signature, Type, TypeError},
    value::{Value, native::NativeFunction},
};

use crate::passes::type_check_expr::Scope;

pub fn parse_module(tokens: &[InfoToken]) -> Result<Module, InfoParseError> {
    let mut module = Module {
        objects: HashMap::new(),
    };

    let mut declarations = HashMap::new();

    let mut ins = Instantiator::new();

    let mut i = 0;

    while i < tokens.len() {
        match tokens[i].token.clone() {
            Token::Keyword(Keyword::Fn) => {
                i += 1;
                let ((name, name_idx), args, signature) =
                    expect_function_signature(&module, tokens, &mut ins, &mut i)?;
                declarations.insert(name.clone(), Declaration::Constant);

                let body = expect_block_or_expr(tokens, &mut i)?;

                let mut last_var = signature.args.len();

                let mut function = Function {
                    ir: vec![Block {
                        terminal: Terminal::Return(last_var),
                        statements: Vec::new(),
                    }],
                    exported: true,
                };

                let global_scope = ins.global_scope();
                let mut scope = global_scope.sub();

                let mut locals = HashMap::new();
                for (idx, arg) in args.iter().enumerate() {
                    locals.insert(arg.clone(), Declaration::Variable(idx));
                    scope.insert(arg.clone(), signature.args[idx]);
                }

                ins.insert(
                    &name,
                    Type::Concrete(ConcreteType::Function(Box::new(signature.clone()))),
                );

                let body_type =
                    infer_expr_type(&body, &mut ins, &mut scope, signature.returns).unwrap();

                if !ins.compatible(body_type, signature.returns) {
                    panic!(
                        "incorrect function return type expected {:?} got {:?}",
                        ins.get_type(signature.returns),
                        ins.get_type(body_type)
                    );
                    todo!("proper error for function body type mismatch")
                }

                to_ir(
                    &mut function,
                    &mut 0,
                    &mut module,
                    body,
                    Some(last_var),
                    &mut declarations,
                    &mut locals,
                    &mut last_var,
                    true,
                )?;

                if let Some(_) = module
                    .objects
                    .insert(name.to_string(), Value::new(function))
                {
                    return Err(InfoParseError {
                        idx: name_idx,
                        error: ParseError::DuplicateName,
                    });
                }
            }
            Token::Keyword(Keyword::Struct) => {
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
                        fields.insert(name.clone(), ins.instantiate(&parse_type(typ)?.expr));
                    }
                }
                i += 1;

                ins.insert(name, Type::Concrete(ConcreteType::Struct(fields)));
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
                    });
                };

                i += 1;
                i += 1;

                let ((name, name_idx), _, signature) =
                    expect_function_signature(&module, tokens, &mut ins, &mut i)?;

                if tokens[i].token != Token::Semicolon {
                    return Err(InfoParseError {
                        idx: tokens[i].idx,
                        error: ParseError::ExpectedSemicolon(tokens[i].clone()),
                    });
                }
                i += 1;

                declarations.insert(name.clone(), Declaration::Constant);
                if let Some(_v) = module.objects.insert(
                    name.clone(),
                    Value::new(NativeFunction {
                        lib_name,
                        func_name: name.clone(),
                    }),
                ) {
                    return Err(InfoParseError {
                        idx: name_idx,
                        error: ParseError::DuplicateName,
                    });
                }

                ins.insert(
                    &name,
                    Type::Concrete(ConcreteType::Function(Box::new(signature))),
                );
            }
            _tk => {
                return Err(InfoParseError {
                    idx: tokens[i].idx,
                    error: ParseError::ExpectedTopLevel,
                });
            }
        }
    }

    Ok(module)
}

pub fn expect_function_signature(
    module: &Module,
    tokens: &[InfoToken],
    ins: &mut Instantiator,
    i: &mut usize,
) -> Result<((String, usize), Vec<String>, Signature), InfoParseError> {
    if let Token::Name(name) = &tokens[*i].token {
        let name_idx = tokens[*i].idx;
        *i += 1;

        let mut args = Vec::new();

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
                    let typ = ins.instantiate(&parse_type(typ)?.expr);
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

            ins.instantiate(&parse_type(&tokens[start..*i])?.expr)
        } else {
            ins.concrete(ConcreteType::Tuple(Vec::new()))
        };
        Ok((
            (name.clone(), name_idx),
            args.iter().map(|arg| arg.0.clone()).collect(),
            Signature {
                args: args.iter().map(|arg| arg.1.clone()).collect(),
                returns,
            },
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
) -> Result<InfoExpr, InfoParseError> {
    if let Some(InfoToken {
        token: Token::Braces(_),
        idx: _,
    }) = tokens.get(*i)
    {
        *i += 1;
        return parse_expression(&tokens[*i - 1..*i]);
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
        return parse_expression(&out);
    }
}
