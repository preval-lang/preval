#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    Let,
    Return,
    Fn,
}

impl TryFrom<&str> for Keyword {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "let" => Ok(Keyword::Let),
            "return" => Ok(Keyword::Return),
            "fn" => Ok(Keyword::Fn),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Dot,
    Call(Vec<Vec<InfoToken>>),
    Assign,
}

impl Operator {
    pub fn precidence(&self) -> i32 {
        match self {
            Operator::Dot => 0,
            Operator::Call(_) => 1,
            Operator::Assign => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    String(String),
    Number(u8),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Name(String),
    Keyword(Keyword),
    Operator(Operator),
    Literal(Literal),
    Parens(Vec<InfoToken>),
    Block(Vec<InfoToken>),
    Semicolon,
    Colon,
    Comma,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InfoToken {
    pub token: Token,
    pub idx: usize,
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

    let mut should_call = false;

    loop {
        match input.chars().nth(i) {
            None => break,
            Some(c) if c.is_alphabetic() || c == '_' => {
                out.push(read_name(input, &mut i, offset));
                should_call = true;
            }
            Some('.') => {
                out.push(InfoToken {
                    token: Token::Operator(Operator::Dot),
                    idx: offset + i,
                });
                i += 1;
                should_call = false;
            }
            Some('=') => {
                out.push(InfoToken {
                    token: Token::Operator(Operator::Assign),
                    idx: offset + i,
                });
                i += 1;
                should_call = false;
            }
            Some(';') => {
                out.push(InfoToken {
                    token: Token::Semicolon,
                    idx: offset + i,
                });
                i += 1;
                should_call = false;
            }
            Some(':') => {
                out.push(InfoToken {
                    token: Token::Colon,
                    idx: offset + i,
                });
                i += 1;
                should_call = false;
            }
            Some(',') => {
                out.push(InfoToken {
                    token: Token::Comma,
                    idx: offset + i,
                });
                i += 1;
                should_call = false;
            }
            Some('(') => {
                if should_call {
                    let read_call = read_parens(input, &mut i, offset, '(', ')')?;
                    if let Token::Parens(tokens) = read_call.token {
                        let mut param_tokens: Vec<Vec<InfoToken>> = Vec::new();
                        for token in tokens {
                            if token.token != Token::Comma {
                                if let Some(last) = param_tokens.last_mut() {
                                    last.push(token);
                                } else {
                                    param_tokens.push(vec![token]);
                                }
                            } else {
                                param_tokens.push(Vec::new());
                            }
                        }

                        match param_tokens.last() {
                            Some(tks) if tks.is_empty() => {
                                param_tokens.remove(param_tokens.len() - 1);
                            }
                            _ => {}
                        }

                        out.push(InfoToken {
                            token: Token::Operator(Operator::Call(param_tokens)),
                            idx: read_call.idx,
                        });
                    } else {
                        unreachable!("read_parens returned non-parens");
                    }
                } else {
                    let parens = read_parens(input, &mut i, offset, '(', ')')?;
                    out.push(parens);
                }
                should_call = true;
            }
            Some('{') => {
                let parens = read_parens(input, &mut i, offset, '{', '}')?;
                out.push(match parens {
                    InfoToken {
                        token: Token::Parens(contents),
                        idx,
                    } => InfoToken {
                        token: Token::Block(contents),
                        idx,
                    },
                    _ => unreachable!("read_parens returned non-parens"),
                });

                should_call = true;
            }
            Some('"') => {
                out.push(read_string(input, &mut i, offset)?);
                should_call = true;
            }
            Some(c) if c.is_numeric() => {
                out.push(read_number(input, &mut i, offset)?);
                should_call = true;
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
                token: if let Ok(num) = number.parse::<u8>() {
                    Token::Literal(Literal::Number(num))
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

fn read_parens(
    input: &str,
    i: &mut usize,
    offset: usize,
    open: char,
    close: char,
) -> Result<InfoToken, TokeniseErrorInfo> {
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
                    return Ok(InfoToken {
                        idx: offset + start,
                        token: Token::Parens(tokenise(&contents, offset + start + 1)?),
                    });
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
