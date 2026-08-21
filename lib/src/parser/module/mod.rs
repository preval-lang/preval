mod expect_body;
mod expect_function_signature;
mod expect_name_and_generics;
use crate::{
	error::{Error, InfoError},
	parser::{
		expression::{InfoParseError, ParseError},
		typ::{InfoTypeExpr, parse_type},
		utility::read_punctuated,
	},
	tokeniser::{InfoToken, Keyword, Literal, Token},
	typ::{GenericImplementation, Instantiator, Template, Trait, TraitMemberExpr, Type, TypeExpr},
	value::native::NativeFunction,
};
use expect_body::expect_body;
use expect_function_signature::expect_function_signature;
use expect_name_and_generics::expect_name_and_generics;
use std::{collections::HashMap, usize};

#[derive(Debug, Clone)]
pub struct Module<'a> {
	pub templates: HashMap<String, Template<'a>>,
	pub methods: HashMap<String, HashMap<String, Template<'a>>>,
	pub path: Vec<String>,
}

pub fn declaration_pass<'a>(
	tokens: &[InfoToken<'a>],
	module: &mut Module<'a>,
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

				module.templates.insert(
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

				let body = expect_body(&tokens, &mut i, &signature.generics)?;

				module.templates.insert(
					signature.name,
					Template {
						expr: InfoTypeExpr {
							expr: TypeExpr::Function(
								signature.arg_types,
								Box::new(signature.return_type.clone()),
								GenericImplementation::Normal(Box::new(body.clone())),
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
				let (name, generics) = expect_name_and_generics(tokens, &mut i)?;
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

				module.templates.insert(
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
				module.templates.insert(
					signature.name.clone(),
					Template {
						expr: InfoTypeExpr {
							expr: TypeExpr::Function(
								signature.arg_types,
								Box::new(signature.return_type),
								GenericImplementation::Native(NativeFunction {
									lib_name,
									func_name: signature.name,
								}),
								signature.args,
							),
							idx: signature.name_idx,
						},
						parameters: signature.generics.len(),
					},
				);
			}
			Token::Keyword(Keyword::Impl) => {
				i += 1;

				let (name, _) = expect_name_and_generics(tokens, &mut i)?;

				let braces = if let Some(InfoToken {
					token: Token::Braces(contents),
					span: _,
				}) = tokens.get(i)
				{
					contents
				} else {
					return Err(InfoParseError {
						error: ParseError::ExpectedExpression(vec![tokens[i].clone()]),
						span: tokens[i].span.clone(),
					}
					.into());
				};
				i += 1;

				{
					let mut i = 0;
					while i < braces.len() {
						match braces[i].token {
							Token::Keyword(Keyword::Fn) => {
								i += 1;
								let sig = expect_function_signature(braces, &mut i)?;
								let body = expect_body(braces, &mut i, &sig.generics)?;

								if !module.methods.contains_key(&name) {
									module.methods.insert(name.clone(), HashMap::new());
								}
								if let Some(_) = module.methods.get_mut(&name).unwrap().insert(
									sig.name.clone(),
									Template {
										expr: InfoTypeExpr {
											expr: TypeExpr::Function(
												sig.arg_types,
												Box::new(sig.return_type),
												GenericImplementation::Normal(Box::new(body)),
												sig.args,
											),
											idx: sig.name_idx,
										},
										parameters: sig.generics.len(),
									},
								) {
									panic!("Method {:?}", sig.name)
								}
							}
							_ => panic!("expected fn"),
						}
					}
				}
			}
			Token::Keyword(Keyword::Trait) => {
				i += 1;

				let name_span = tokens[i].span.clone();
				let (name, generics) = expect_name_and_generics(tokens, &mut i)?;

				let braces = if let Token::Braces(contents) = &tokens[i].token {
					contents
				} else {
					return Err(InfoError {
						span: tokens[i].span.clone(),
						data: Error::ParseError(ParseError::ExpectedTopLevel),
					});
				};
				i += 1;

				let mut members = HashMap::new();

				{
					let mut i = 0;

					while i < braces.len() {
						match braces[i].token {
							Token::Keyword(Keyword::Fn) => {
								i += 1;
								let sig = expect_function_signature(braces, &mut i)?;
								if tokens[i].token != Token::Semicolon {
									return Err(InfoError {
										span: tokens[i].span.clone(),
										data: Error::ParseError(ParseError::ExpectedSemicolon(
											tokens[i].clone(),
										)),
									});
								}
								i += 1;

								members.insert(
									sig.name,
									TraitMemberExpr::Function {
										generics: sig.generics,
										args: sig.arg_types,
										returns: sig.return_type,
									},
								);
							}
							Token::Keyword(Keyword::Type) => {
								i += 1;
								let name = if let Token::Name(name) = &tokens[i].token {
									name
								} else {
									return Err(InfoError {
										span: tokens[i].span.clone(),
										data: Error::ParseError(ParseError::ExpectedName),
									});
								};
								i += 1;

								members.insert(name.clone(), TraitMemberExpr::AssociatedType);
							}
							_ => panic!("expected fn"),
						}
					}
				}

				module.templates.insert(
					name,
					Template {
						parameters: generics.len(),
						expr: InfoTypeExpr {
							expr: TypeExpr::Trait(Trait {
								members,
								impls: HashMap::new(),
							}),
							idx: name_span,
						},
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

pub fn implementation_pass<'a>(
	ins: &mut Instantiator<'a>,
	subtype: Option<usize>,
) -> Result<(), InfoError<'a>> {
	let ns = if let Some(subtype) = subtype {
		&ins.subtype_members.get(&subtype).unwrap().templates
	} else {
		&ins.global_namespace.templates
	};

	for name in ns.keys().map(ToOwned::to_owned).collect::<Vec<_>>() {
		let ns = if let Some(subtype) = subtype {
			&ins.subtype_members.get(&subtype).unwrap().templates
		} else {
			&ins.global_namespace.templates
		};
		let params = ns.get(&name).unwrap().parameters;
		let generics = (0..params)
			.map(|i| ins.add(Type::Placeholder(i)))
			.collect::<Vec<_>>();

		let ns = if let Some(subtype) = subtype {
			&ins.subtype_members.get(&subtype).unwrap().templates
		} else {
			&ins.global_namespace.templates
		};

		let expr = ns.get(&name).unwrap().expr.clone();
		let t = ins.instantiate(&expr, &generics)?;
		if ins.subtype_members.contains_key(&t) {
			implementation_pass(ins, Some(t))?;
		}
	}

	Ok(())
}
