use std::fmt::Debug;

use crate::{
    typ::{self, TypeExpr, type_names},
    value::Value,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    Let,
    Return,
    Fn,
    If,
    Else,
    Bool(bool),
    Struct,
    Dylib,
    Guard,
    Is,
}

impl TryFrom<&str> for Keyword {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "let" => Ok(Keyword::Let),
            "return" => Ok(Keyword::Return),
            "fn" => Ok(Keyword::Fn),
            "if" => Ok(Keyword::If),
            "else" => Ok(Keyword::Else),
            "true" => Ok(Keyword::Bool(true)),
            "false" => Ok(Keyword::Bool(false)),
            "struct" => Ok(Keyword::Struct),
            "dylib" => Ok(Keyword::Dylib),
            "guard" => Ok(Keyword::Guard),
            "is" => Ok(Keyword::Is),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Name(String),
    Keyword(Keyword),
    Literal(Literal),
    Parens(Vec<InfoToken>),
    Braces(Vec<InfoToken>),
    Index(Vec<InfoToken>),
    Semicolon,
    Colon,
    Comma,
    Dot,
    Assignment,
    Union,
    LessThan,
    GreaterThan,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Bool(bool),
    String(String),
    Usize(usize),
}

impl Literal {
    pub fn get_type(&self) -> TypeExpr {
        match self {
            Literal::Bool(_) => type_names::bool(),
            Literal::String(_) => type_names::string(),
            Literal::Usize(_) => type_names::usize(),
        }
    }

    pub fn to_value(self) -> Value {
        let typ = self.get_type();
        match self {
            Literal::Bool(b) => Value::new(b, typ),
            Literal::String(s) => Value::new(s, typ),
            Literal::Usize(u) => Value::new(u, typ),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct InfoToken {
    pub token: Token,
    pub idx: usize,
}

impl Debug for InfoToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Token::fmt(&self.token, f)
    }
}

#[derive(Debug)]
pub struct TokeniseErrorInfo {
    pub idx: usize,
    pub error: TokeniseError,
}

#[derive(Debug)]
pub enum TokeniseError {
    UnclosedParens,
    UnclosedQuotes,
    ExpectedToken(char),
    ExpectedNumber(String),
}

#[derive(Debug)]
pub struct EOF {}

pub fn get_line_and_column(input: &str, idx: usize) -> Result<(usize, usize), EOF> {
    let mut col = 1;
    let mut line = 1;

    for (i, c) in input.char_indices() {
        if i == idx {
            return Ok((line, col));
        }
        if c == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }

    Err(EOF {})
}

pub fn tokenise(input: &str, offset: usize) -> Result<Vec<InfoToken>, TokeniseErrorInfo> {
    let mut out = Vec::new();

    let mut i = 0;

    loop {
        match input.chars().nth(i) {
            None => break,
            Some(c) if c.is_alphabetic() || c == '_' => {
                out.push(read_name(input, &mut i, offset));
            }
            Some('.') => {
                out.push(InfoToken {
                    token: Token::Dot,
                    idx: offset + i,
                });
                i += 1;
            }
            Some('=') => {
                out.push(InfoToken {
                    token: Token::Assignment,
                    idx: offset + i,
                });
                i += 1;
            }
            Some(';') => {
                out.push(InfoToken {
                    token: Token::Semicolon,
                    idx: offset + i,
                });
                i += 1;
            }
            Some(':') => {
                out.push(InfoToken {
                    token: Token::Colon,
                    idx: offset + i,
                });
                i += 1;
            }
            Some(',') => {
                out.push(InfoToken {
                    token: Token::Comma,
                    idx: offset + i,
                });
                i += 1;
            }
            Some('|') => {
                out.push(InfoToken {
                    token: Token::Union,
                    idx: offset + i,
                });
                i += 1;
            }
            Some('<') => {
                out.push(InfoToken {
                    token: Token::LessThan,
                    idx: offset + i,
                });
                i += 1;
            }
            Some('>') => {
                out.push(InfoToken {
                    token: Token::GreaterThan,
                    idx: offset + i,
                });
                i += 1;
            }
            Some('(') => {
                let (idx, contents) = read_brackets(input, &mut i, offset, '(', ')')?;
                out.push(InfoToken {
                    token: Token::Parens(contents),
                    idx,
                });
            }
            Some('{') => {
                let (idx, contents) = read_brackets(input, &mut i, offset, '{', '}')?;
                out.push(InfoToken {
                    token: Token::Braces(contents),
                    idx,
                });
            }
            Some('[') => {
                let (idx, contents) = read_brackets(input, &mut i, offset, '[', ']')?;
                out.push(InfoToken {
                    token: Token::Index(contents),
                    idx,
                });
            }
            Some('"') => {
                out.push(read_string(input, &mut i, offset)?);
            }
            Some(c) if c.is_numeric() => {
                out.push(read_number(input, &mut i, offset)?);
            }
            Some(c) if c.is_whitespace() => i += 1,
            Some(a) => {
                return Err(TokeniseErrorInfo {
                    idx: offset + i,
                    error: TokeniseError::ExpectedToken(a),
                });
            }
        }
    }

    Ok(out)
}

fn read_number(input: &str, i: &mut usize, offset: usize) -> Result<InfoToken, TokeniseErrorInfo> {
    let start = *i;

    let mut number = String::new();

    loop {
        let c = input.chars().nth(*i);
        if c.is_none() || !(c.unwrap().is_numeric() || c.unwrap() == '_') {
            return Ok(InfoToken {
                idx: offset + start,
                token: if let Ok(num) = number.parse::<usize>() {
                    Token::Literal(Literal::Usize(num))
                } else {
                    return Err(TokeniseErrorInfo {
                        idx: offset + start,
                        error: TokeniseError::ExpectedNumber(number),
                    });
                },
            });
        }
        number.push(c.unwrap());
        *i += 1;
    }
}

fn read_name(input: &str, i: &mut usize, offset: usize) -> InfoToken {
    let start = *i;

    let mut name = String::new();

    loop {
        let c = input.chars().nth(*i);
        if c.is_none() || !(c.unwrap().is_alphanumeric() || c.unwrap() == '_') {
            return InfoToken {
                idx: offset + start,
                token: if let Ok(keyword) = Keyword::try_from(name.as_str()) {
                    Token::Keyword(keyword)
                } else {
                    Token::Name(name)
                },
            };
        }
        name.push(c.unwrap());
        *i += 1;
    }
}

fn read_brackets(
    input: &str,
    i: &mut usize,
    offset: usize,
    open: char,
    close: char,
) -> Result<(usize, Vec<InfoToken>), TokeniseErrorInfo> {
    let start = *i;

    let mut contents = String::new();

    let mut open_parens = 0;

    loop {
        let c = input.chars().nth(*i);
        match c {
            Some(c) if c == open => {
                open_parens += 1;
                if open_parens != 1 {
                    contents.push(open);
                }
            }
            Some(c) if c == close => {
                open_parens -= 1;
                if open_parens == 0 {
                    *i += 1;
                    return Ok((offset + start, (tokenise(&contents, offset + start + 1)?)));
                } else {
                    contents.push(close);
                }
            }
            Some(c) => {
                contents.push(c);
            }
            None => {
                return Err(TokeniseErrorInfo {
                    idx: start + offset,
                    error: TokeniseError::UnclosedParens,
                });
            }
        }
        *i += 1;
    }
}

fn read_string(input: &str, i: &mut usize, offset: usize) -> Result<InfoToken, TokeniseErrorInfo> {
    // TODO: escape sequences

    let start = *i;

    let mut contents = String::new();

    *i += 1;

    loop {
        let c = input.chars().nth(*i);
        match c {
            Some('"') => {
                *i += 1;
                return Ok(InfoToken {
                    idx: offset + start,
                    token: Token::Literal(Literal::String(contents)),
                });
            }
            Some(c) => {
                contents.push(c);
            }
            None => {
                return Err(TokeniseErrorInfo {
                    idx: start + offset,
                    error: TokeniseError::UnclosedQuotes,
                });
            }
        }
        *i += 1;
    }
}
