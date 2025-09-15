use std::{collections::HashMap, usize};

use crate::{
    expression_parser::{InfoExpr, InfoParseError, ParseError, parse_expression},
    ir::{Block, Declaration, Function, Module, Terminal, to_ir},
    tokeniser::{InfoToken, Keyword, Operator, Token},
    typ::{Signature, Type, get_type},
};

pub fn parse_module<VarRepr: Clone>(
    tokens: &[InfoToken],
) -> Result<Module<VarRepr>, InfoParseError> {
    let mut module = Module {
        constants: Vec::new(),
        functions: HashMap::new(),
    };

    let mut declarations = HashMap::new();

    declarations.insert(
        "print".to_string(),
        Declaration::Function(Signature {
            args: vec![Type::IO, Type::Slice(Box::new(Type::u8))],
            returns: Type::Tuple(Vec::new()),
        }),
    );

    let mut i = 0;

    while i < tokens.len() {
        match tokens[i].token.clone() {
            Token::Keyword(Keyword::Fn) => {
                i += 1;
                if let Token::Name(name) = &tokens[i].token {
                    i += 1;

                    let mut args = Vec::new();

                    if let Token::Operator(Operator::Call(arg_colon_types)) = &tokens[i].token {
                        for arg_colon_type in arg_colon_types {
                            if let [
                                InfoToken {
                                    token: Token::Name(name),
                                    idx: name_idx,
                                },
                                InfoToken {
                                    token: Token::Colon,
                                    idx: colon_idx,
                                },
                                typ @ ..,
                            ] = arg_colon_type.as_slice()
                            {
                                args.push((name, get_type(typ, &mut 0)?));
                            }
                        }
                        i += 1;
                    } else {
                        panic!("Missing function parameters, got {:?}", tokens[i]);
                    }
                    let mut returns = Type::Tuple(Vec::new());
                    if let Token::Colon = &tokens[i].token {
                        i += 1;
                        returns = get_type(tokens, &mut i)?;
                    }
                    let signature = Signature {
                        args: args.iter().map(|arg| arg.1.clone()).collect(),
                        returns,
                    };
                    declarations.insert(name.clone(), Declaration::Function(signature.clone()));

                    let body = expect_block_or_expr(tokens, &mut i)?;

                    let next_var = args.len();

                    let mut function = Function {
                        ir: vec![Block {
                            terminal: Terminal::Return(Some(next_var)),
                            statements: Vec::new(),
                        }],
                        exported: true,
                        variable_types: HashMap::new(),
                        signature,
                    };

                    let mut locals = HashMap::new();
                    for (idx, arg) in args.iter().enumerate() {
                        function.variable_types.insert(idx, arg.1.clone());
                        locals.insert(arg.0.clone(), Declaration::Variable(idx));
                    }

                    to_ir(
                        &mut function,
                        &mut 0,
                        &mut module,
                        body,
                        Some(next_var),
                        &mut declarations,
                        &mut locals,
                    )?;

                    if let Some(_) = module.functions.insert(name.to_string(), function) {
                        return Err(InfoParseError {
                            idx: tokens[i].idx,
                            error: ParseError::DuplicateName,
                        });
                    }
                } else {
                    return Err(InfoParseError {
                        idx: tokens[i].idx,
                        error: ParseError::ExpectedName,
                    });
                }
            }
            tk => {
                return Err(InfoParseError {
                    idx: tokens[i].idx,
                    error: ParseError::ExpectedTopLevel,
                });
            }
        }
    }

    Ok(module)
}

pub fn expect_block_or_expr(
    tokens: &[InfoToken],
    i: &mut usize,
) -> Result<InfoExpr, InfoParseError> {
    if let Some(InfoToken {
        token: Token::Block(_),
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
