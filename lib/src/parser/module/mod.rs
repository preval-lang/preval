mod expect_body;
mod expect_function_signature;
mod expect_generics;
mod expect_impl_block;
mod expect_name_and_generics;
mod expect_token;
mod expect_use;
use crate::{
	error::{Error, InfoError},
	parser::{
		expression::{InfoParseError, ParseError},
		module::{
			expect_impl_block::{ImplBlock, expect_impl_block},
			expect_use::expect_use,
		},
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
	pub impls: Vec<ImplBlock<'a>>,
}

pub fn declaration_pass<'a>(
	tokens: &[InfoToken<'a>],
	module: &mut Module<'a>,
) -> Result<(), InfoError<'a>> {
	let mut i = 0;

	while i < tokens.len() {
		match tokens[i].token.clone() {
			Token::Keyword(Keyword::Use) => {
				let use_d = expect_use(tokens, &mut i)?;
				module.templates.insert(
					use_d.name,
					Template {
						parameters: 0,
						expr: use_d.typ,
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
				let impl_block = expect_impl_block(tokens, &mut i)?;
				module.impls.push(impl_block);
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
								if braces[i].token != Token::Semicolon {
									return Err(InfoError {
										span: braces[i].span.clone(),
										data: Error::ParseError(ParseError::ExpectedSemicolon(
											braces[i].clone(),
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
								let name = if let Token::Name(name) = &braces[i].token {
									i += 1;
									name
								} else {
									return Err(InfoError {
										span: braces[i].span.clone(),
										data: Error::ParseError(ParseError::ExpectedName),
									});
								};
								if let Token::Semicolon = braces[i].token {
									i += 1;
								} else {
									panic!("expected semicolon, got {:?}", braces[i]);
								}
								members.insert(name.clone(), TraitMemberExpr::AssociatedType);
							}
							ref tk => panic!("expected fn or type, got {tk:?}"),
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

fn namespace_templates<'a, 'b>(
	ins: &'b Instantiator<'a>,
	subtype: Option<usize>,
) -> &'b HashMap<String, Template<'a>> {
	match subtype {
		Some(subtype) => &ins.subtype_members.get(&subtype).unwrap().templates,
		None => &ins.global_namespace.templates,
	}
}

pub fn implementation_pass<'a>(
	ins: &mut Instantiator<'a>,
	subtype: Option<usize>,
) -> Result<(), InfoError<'a>> {
	let names_and_params = namespace_templates(ins, subtype)
		.iter()
		.map(|(name, template)| (name.clone(), template.parameters))
		.collect::<Vec<_>>();

	for (name, params) in names_and_params {
		let generics = (0..params)
			.map(|i| ins.add(Type::Placeholder(i)))
			.collect::<Vec<_>>();

		let expr = namespace_templates(ins, subtype)
			.get(&name)
			.unwrap()
			.expr
			.clone();
		let t = ins.instantiate(&expr, &generics)?;
		if ins.subtype_members.contains_key(&t) {
			implementation_pass(ins, Some(t))?;
		}
	}

	Ok(())
}
