use std::{collections::HashMap, usize};

use crate::{
    expression_parser::{InfoExpr, InfoParseError, ParseError, parse_expression},
    tokeniser::{InfoToken, Keyword, Literal, Operator, Token},
    typ::{Signature, Type, get_type},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub signature: Signature,
    pub body: InfoExpr,
    pub exported: bool,
}

#[derive(Debug)]
pub struct Module {
    pub constants: HashMap<String, (Literal, bool)>,
}

pub fn parse_module(tokens: &[InfoToken]) -> Result<Module, InfoParseError> {
    let mut module = Module {
        constants: HashMap::new(),
    };

    let mut declarations = HashMap::new();

    let mut i = 0;

    while i < tokens.len() {
        match tokens[i].token.clone() {
            Token::Keyword(Keyword::Fn) => {
                i += 1;
                if let Token::Name(name) = &tokens[i].token {
                    i += 1;

                    let mut args = Vec::new();

                    if let Token::Operator(Operator::Call(arg_colon_types, generics)) =
                        &tokens[i].token
                    {
                        if generics.len() != 0 {
                            todo!("GENERICS at module parser")
                        }
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
                        generics: Vec::new(),
                    };
                    declarations.insert(name.clone(), Type::Function(Box::new(signature.clone())));

                    let body = expect_block_or_expr(tokens, &mut i)?;

                    let next_var = args.len();

                    if let Some(_) = module.constants.insert(
                        name.to_string(),
                        Literal::Function(Box::new(Function {
                            body: body,
                            signature,
                            exported: true,
                        })),
                    ) {
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
