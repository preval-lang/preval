use std::{collections::HashMap, usize};

use crate::{
	error::{InfoError, Span},
	parser::{
		expression::{InfoExpr, InfoParseError, ParseError, parse_expression},
		typ::{InfoTypeExpr, parse_type},
		utility::read_punctuated,
	},
	tokeniser::{InfoToken, Keyword, Literal, Token},
	typ::{GenericImplementation, Template, TypeExpr},
	value::native::NativeFunction,
};

pub fn add_prefix(prefix: &[String], name: String) -> Vec<String> {
	let mut v = prefix.to_vec();
	v.push(name);
	v
}

#[derive(Debug)]
pub enum Symbol<'a> {
	Fn(Signature<'a>, InfoExpr<'a>),
	DylibFn(Signature<'a>),
	Struct(Vec<String>, HashMap<String, InfoTypeExpr<'a>>),
	Alias,
}

#[derive(Clone, Debug)]
pub struct Signature<'a> {
	name: String,
	name_idx: Span<'a>,
	generics: Vec<String>,
	args: Vec<String>,
	arg_types: Vec<InfoTypeExpr<'a>>,
	return_type: InfoTypeExpr<'a>,
}

pub fn declaration_pass<'a>(
	tokens: &[InfoToken<'a>],
	module: &mut HashMap<String, Template<'a>>,
) -> Result<(), InfoError<'a>> {
	let mut i = 0;

	while i < tokens.len() {
		match tokens[i].token.clone() {
			Token::Keyword(Keyword::Use) => {
				i += 1;
				let start = i;
				while tokens[i].token != Token::Semicolon {
					i += 1;
				}
				let end = i;
				let items = read_punctuated(&tokens[start..end], Token::DoubleColon)?;
				i += 1;

				let mut path = Vec::new();

				for item in items {
					if let [
						InfoToken {
							token: Token::Name(name),
							span: _,
						},
					] = &item[..]
					{
						path.push(name.clone());
					} else {
						return Err(InfoParseError {
							error: ParseError::ExpectedName,
							span: item[0].span.clone(),
						}
						.into());
					}
				}

				module.insert(
					path.last().unwrap().clone(),
					Template {
						expr: parse_type(&tokens[start..end], &vec![])?,
						parameters: 0,
					},
				);
			}
			Token::Keyword(Keyword::Fn) => {
				i += 1;
				let signature = expect_function_signature(&tokens, &mut i)?;

				let body = expect_block_or_expr(&tokens, &mut i, &signature.generics)?;

				module.insert(
					signature.name,
					Template {
						expr: InfoTypeExpr {
							expr: TypeExpr::Function(
								signature.arg_types,
								Box::new(signature.return_type.clone()),
								Some(GenericImplementation::Normal(Box::new(body.clone()))),
								signature.args,
							),
							idx: signature.name_idx,
						},
						parameters: signature.generics.len(),
					},
				);
			}
			Token::Keyword(Keyword::Struct) => {
				let idx = i;
				i += 1;
				let name = if let Token::Name(name) = &tokens[i].token {
					Ok(name)
				} else {
					Err(InfoParseError {
						span: tokens[i].span.clone(),
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
							span: _,
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
									span: _,
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
						span: tokens[i].span.clone(),
						error: ParseError::ExpectedExpression(tokens[i..].to_vec()),
					})
				}?;

				let mut fields = HashMap::new();

				for field_colon_type in read_punctuated(block, Token::Comma)? {
					if let [
						InfoToken {
							token: Token::Name(name),
							span: _name_idx,
						},
						InfoToken {
							token: Token::Colon,
							span: _colon_idx,
						},
						typ @ ..,
					] = field_colon_type.as_slice()
					{
						fields.insert(name.clone(), parse_type(typ, &generics)?);
					}
				}
				i += 1;

				module.insert(
					name.clone(),
					Template {
						expr: InfoTypeExpr {
							expr: TypeExpr::Struct(fields.clone()),
							idx: tokens[idx].span.clone(),
						},
						parameters: generics.len(),
					},
				);
			}
			Token::Keyword(Keyword::Dylib) => {
				i += 1;
				let lib_name = if let InfoToken {
					span: _,
					token: Token::Literal(Literal::String(s)),
				} = &tokens[i]
				{
					s.clone()
				} else {
					return Err(InfoParseError {
						span: tokens[i].span.clone(),
						error: ParseError::ExpectedString(tokens[i].clone()),
					}
					.into());
				};

				i += 1;
				i += 1;

				let signature = expect_function_signature(&tokens, &mut i)?;

				if tokens[i].token != Token::Semicolon {
					return Err(InfoParseError {
						span: tokens[i].span.clone(),
						error: ParseError::ExpectedSemicolon(tokens[i].clone()),
					}
					.into());
				}
				i += 1;
				module.insert(
					signature.name.clone(),
					Template {
						expr: InfoTypeExpr {
							expr: TypeExpr::Function(
								signature.arg_types,
								Box::new(signature.return_type),
								Some(GenericImplementation::Native(NativeFunction {
									lib_name,
									func_name: signature.name,
								})),
								signature.args,
							),
							idx: signature.name_idx,
						},
						parameters: signature.generics.len(),
					},
				);
			}
			_tk => {
				return Err(InfoParseError {
					span: tokens[i].span.clone(),
					error: ParseError::ExpectedTopLevel,
				}
				.into());
			}
		}
	}

	Ok(())
}

// pub fn implementation_pass<'a>(
// 	mut instantiator: &mut Instantiator<'a>,
// ) -> Result<(), InfoError<'a>> {
// 	for (name, template) in &instantiator.global_namespace {
// 		match template.expr {
// 			Symbol::Alias => {
// 				instantiator.instantiate(
// 					&InfoTypeExpr {
// 						expr: TypeExpr::Name(fqn.clone(), true, vec![]),
// 						idx: Span {
// 							file: Cow::Owned(file!().into()),
// 							index: 0,
// 						},
// 					},
// 					&vec![],
// 					&fqn[0..fqn.len() - 1],
// 				)?;
// 			}
// 			Symbol::DylibFn(sig) => {
// 				let generics = (0..sig.generics.len())
// 					.map(|i| instantiator.add(Type::Placeholder(i)))
// 					.collect::<Vec<_>>();

// 				for arg in sig.arg_types.clone() {
// 					instantiator.instantiate(&arg, &generics, &fqn[0..fqn.len() - 1])?;
// 				}
// 				instantiator.instantiate(&sig.return_type, &generics, &fqn[0..fqn.len() - 1])?;
// 			}
// 			Symbol::Struct(generics, fields) => {
// 				let generics = (0..generics.len())
// 					.map(|i| instantiator.add(Type::Placeholder(i)))
// 					.collect::<Vec<_>>();

// 				for (_, arg) in fields {
// 					instantiator.instantiate(&arg, &generics, &fqn[0..fqn.len() - 1])?;
// 				}
// 			}
// 			Symbol::Fn(sig, body) => {
// 				let generics = (0..sig.generics.len())
// 					.map(|i| instantiator.add(Type::Placeholder(i)))
// 					.collect::<Vec<_>>();

// 				let return_type_ins = instantiator.instantiate(
// 					&sig.return_type,
// 					&generics,
// 					&fqn[0..fqn.len() - 1],
// 				)?;

// 				let mut scope = Scope::new();

// 				for (idx, arg) in sig.args.iter().enumerate() {
// 					scope.insert(
// 						arg.clone(),
// 						instantiator.instantiate(
// 							&sig.arg_types[idx],
// 							&generics,
// 							&fqn[0..fqn.len() - 1],
// 						)?,
// 					);
// 				}

// 				let body_type = infer_expr_type(
// 					body,
// 					&mut instantiator,
// 					&mut scope,
// 					return_type_ins,
// 					&generics,
// 					&fqn[0..fqn.len() - 1],
// 				)?;

// 				if !instantiator
// 					.compatible(body_type.typ, return_type_ins, 0)
// 					.unwrap()
// 				{
// 					return Err(InfoError {
// 						span: sig.return_type.idx.clone(),
// 						data: Error::TypeError(TypeError::IncompatibleTypes {
// 							expected: instantiator.get_type(return_type_ins).unwrap().clone(),
// 							got: instantiator.get_type(body_type.typ).unwrap().clone(),
// 						}),
// 					});
// 				}
// 			}
// 		}
// 	}

// 	Ok(())
// }

fn expect_function_signature<'a>(
	tokens: &[InfoToken<'a>],
	i: &mut usize,
) -> Result<Signature<'a>, InfoParseError<'a>> {
	if let Token::Name(name) = &tokens[*i].token {
		let name_idx = tokens[*i].span.clone();
		*i += 1;

		let mut args = Vec::new();
		let generics_tokens = if let Token::LessThan = &tokens[*i].token {
			*i += 1;
			let start = *i;
			loop {
				if let Some(InfoToken {
					token: Token::GreaterThan,
					span: _,
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
							span: _,
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
						span: _name_idx,
					},
					InfoToken {
						token: Token::Colon,
						span: _colon_idx,
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
					span: _,
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
				idx: tokens[*i].span.clone(),
			}
		};
		Ok(Signature {
			name: name.clone(),
			name_idx,
			generics,
			args: args.iter().map(|arg| arg.0.clone()).collect(),
			arg_types: args.iter().map(|arg| arg.1.clone()).collect(),
			return_type: returns,
		})
	} else {
		Err(InfoParseError {
			span: tokens[*i].span.clone(),
			error: ParseError::ExpectedFunctionSignature(tokens[*i].clone()),
		})
	}
}

pub fn expect_block_or_expr<'a>(
	tokens: &[InfoToken<'a>],
	i: &mut usize,
	generics: &[String],
) -> Result<InfoExpr<'a>, InfoParseError<'a>> {
	if let Some(InfoToken {
		token: Token::Braces(_),
		span: _,
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
