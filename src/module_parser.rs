use std::{collections::HashMap, usize};

use crate::{
    expression_parser::{InfoExpr, InfoParseError, ParseError, parse_expression},
    ir::{Block, Declaration, Function, Module, Signature, Statement, Terminal, Type, to_ir},
    tokeniser::{InfoToken, Keyword, Token},
};

pub fn parse_module(tokens: &[InfoToken]) -> Result<Module, InfoParseError> {
    let mut module = Module {
        constants: Vec::new(),
        functions: HashMap::new(),
    };

    let mut declarations = HashMap::new();

    declarations.insert(
        "print".to_string(),
        Declaration::Function(Signature {
            args: vec![Type::Slice(Box::new(Type::u8))],
            returns: Type::void,
        }),
    );

    let mut i = 0;

    while i < tokens.len() {
        match tokens[i].token.clone() {
            Token::Keyword(Keyword::Fn) => {
                i += 1;
                if let Token::Name(name) = &tokens[i].token {
                    i += 1;
                    i += 1; // TODO: DONT SKIP PARENS
                    let mut returns = Type::void;
                    if let Token::Colon = &tokens[i].token {
                        i += 1;
                        if let Token::Name(name) = &tokens[i].token {
                            returns = match name.as_str() {
                                // TODO: real type system
                                "stringSlice" => Type::Slice(Box::new(Type::u8)),
                                _ => panic!("Unsupported type"),
                            };
                            i += 1;
                        }
                    }
                    declarations.insert(
                        name.clone(),
                        Declaration::Function(Signature {
                            args: Vec::new(),
                            returns: returns,
                        }),
                    );
                    let body = expect_block_or_expr(tokens, &mut i)?;

                    let mut function = Function {
                        ir: vec![Block {
                            terminal: Terminal::Return(None),
                            statements: Vec::new(),
                        }],
                        exported: true,
                        variable_types: Vec::new(),
                    };

                    let mut next_var = 0;

                    if let Err(e) = to_ir(
                        &mut function,
                        0,
                        &mut module,
                        body,
                        &mut next_var,
                        true,
                        &mut declarations,
                        true,
                    ) {
                        println!("TODO: introduce AST modules");
                        panic!("{e:?}");
                    }

                    next_var += 1; // unnecessary as of now but lets just maintain the count for sanity purposes

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
