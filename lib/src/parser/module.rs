use crate::{
    ir::{Block, Declaration, Function, Module, Terminal, to_ir},
    parser::{
        expression::{InfoExpr, InfoParseError, ParseError, parse_expression},
        utility::read_punctuated,
    },
    tokeniser::{InfoToken, Keyword, Token},
    value::{
        Value,
        native::NativeFunction,
        typ::{Signature, Type},
    },
};
use std::{collections::HashMap, usize};

pub fn parse_module(tokens: &[InfoToken]) -> Result<Module, InfoParseError> {
    let mut module = Module {
        objects: HashMap::new(),
    };

    let mut declarations = HashMap::new();

    let mut i = 0;

    while i < tokens.len() {
        match tokens[i].token.clone() {
            Token::Keyword(Keyword::Fn) => {
                let ((name, name_idx), args) = expect_function_signature(&module, tokens, &mut i)?;
                declarations.insert(name.clone(), Declaration::Constant);

                let body = expect_block_or_expr(tokens, &mut i)?;

                let mut last_var = args.len();

                let mut function = Function {
                    ir: vec![Block {
                        terminal: Terminal::Return(last_var),
                        statements: Vec::new(),
                    }],
                    exported: true,
                };

                let mut locals = HashMap::new();
                for (idx, arg) in args.iter().enumerate() {
                    locals.insert(arg.clone(), Declaration::Variable(idx));
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
            Token::Keyword(Keyword::Dylib) => {
                i += 1;
                let lib_name = if let InfoToken {
                    idx: _,
                    token:
                        Token::Literal(Value {
                            typ: Type::String,
                            data,
                        }),
                } = &tokens[i]
                {
                    data.as_any().downcast_ref::<String>().unwrap().clone()
                } else {
                    return Err(InfoParseError {
                        idx: tokens[i].idx,
                        error: ParseError::ExpectedString(tokens[i].clone()),
                    });
                };

                i += 1;

                let ((name, name_idx), _) = expect_function_signature(&module, tokens, &mut i)?;

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
    i: &mut usize,
) -> Result<((String, usize), Vec<String>), InfoParseError> {
    *i += 1;
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
                ] = &arg_colon_type[..]
                {
                    args.push(name.clone());
                }
            }
            *i += 1;
        } else {
            panic!("Missing function parameters, got {:?}", tokens[*i]);
        }
        if let Token::Colon = &tokens[*i].token {
            *i += 1;
        }
        Ok((
            (name.clone(), name_idx),
            args.iter().map(|arg| arg.clone()).collect(),
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
