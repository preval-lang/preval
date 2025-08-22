use crate::{
    expression_parser::{InfoParseError, ParseError},
    ir::Type,
    tokeniser::{InfoToken, Token},
};

pub fn get_type(tokens: &[InfoToken], i: &mut usize) -> Result<Type, InfoParseError> {
    let rv = match &tokens[*i].token {
        Token::Name(name) if name == "stringSlice" => Ok(Type::Slice(Box::new(Type::u8))),
        _ => Err(InfoParseError {
            idx: tokens[*i].idx,
            error: ParseError::TypeUndefined(tokens.to_vec()),
        }),
    };
    *i += 1;
    rv
}
