use std::{borrow::Cow, collections::HashMap};

use crate::{
	parser::{expression::InfoExpr, typ::InfoTypeExpr},
	typ::{ConcreteType, InfoTypeError, Instantiator, Type, TypeError, TypeExpr},
};

use crate::parser::expression::Expr;

#[derive(Debug)]
pub struct Scope<'a> {
	scopes: Vec<Cow<'a, HashMap<String, usize>>>,
}

impl<'a> Scope<'a> {
	pub fn new() -> Self {
		Self {
			scopes: vec![Cow::Owned(HashMap::new())],
		}
	}

	pub fn sub(&'a self) -> Scope<'a> {
		let mut scopes = self.scopes.clone();
		scopes.push(Cow::Owned(HashMap::new()));
		Self { scopes }
	}

	pub fn get(&self, name: &str) -> Option<usize> {
		for scope in self.scopes.iter().rev() {
			if let Some(typ) = (*scope).get(name) {
				return Some(typ.clone());
			}
		}
		None
	}

	pub fn insert(&mut self, name: String, typ: usize) {
		let mut last = self.scopes.pop().unwrap().into_owned();
		last.insert(name, typ);
		self.scopes.push(Cow::Owned(last));
	}
}

#[derive(Debug, Clone)]
pub struct TypedExpr {
	pub typ: usize,
	pub expr: Expr<TypedExpr, usize, String>,
}

pub fn infer_expr_type<'a>(
	expr: InfoExpr<'a>,
	ins: &mut Instantiator<'a>,
	scope: &mut Scope,
	return_type: usize,
	generics: &[usize],
	prefix: &[String],
) -> Result<TypedExpr, InfoTypeError<'a>> {
	match expr.expr {
		Expr::Local(_) => unreachable!("At this stage locals will be Names instead"),
		Expr::Literal(value) => Ok(TypedExpr {
			typ: ins.add(Type::Concrete(value.get_type())),
			expr: Expr::Literal(value.clone()),
		}),
		Expr::Name(name) => {
			if let InfoTypeExpr {
				expr: TypeExpr::Name(name, params),
				idx: _,
			} = &name
			{
				if name.len() == 1 && params.len() == 0 {
					if let Some(type_id) = scope.get(name) {
						return Ok(TypedExpr {
							typ: type_id,
							expr: Expr::Local(name.clone()),
						});
					}
				}
			};
			let typ = ins.instantiate(&name, generics)?;
			Ok(TypedExpr {
				typ: typ.clone(),
				expr: Expr::Name(typ),
			})
		}
		Expr::Block(statements, returns) => {
			let mut scope = scope.sub();
			let mut typed_statements = Vec::new();
			for statement in statements {
				typed_statements.push(infer_expr_type(
					statement,
					ins,
					&mut scope,
					return_type,
					generics,
					prefix,
				)?);
			}

			let typ = if typed_statements.len() == 0 || !returns {
				ins.add(Type::Concrete(ConcreteType::Tuple(Vec::new())))
			} else {
				typed_statements.last().unwrap().typ
			};

			Ok(TypedExpr {
				typ,
				expr: Expr::Block(typed_statements, returns.clone()),
			})
		}
		Expr::InitializeStruct(struct_type_expr, fields) => {
			let struct_type_id = ins.instantiate(&struct_type_expr, generics)?;
			let struct_type = ins.get_type(struct_type_id).unwrap();
			let struct_members = if let Type::Concrete(ConcreteType::Struct(members)) = struct_type
			{
				members.clone()
			} else {
				return Err(InfoTypeError {
					span: struct_type_expr.idx.clone(),
					error: TypeError::NotAStruct(struct_type.clone()),
				});
			};

			if fields.len() != struct_members.len() {
				return Err(InfoTypeError {
					span: expr.idx.clone(),
					error: TypeError::IncorrectFieldCount {
						expected: struct_members.len(),
						got: fields.len(),
					},
				});
			}

			let mut assignees = HashMap::new();

			for (name, field) in fields {
				let assignee = infer_expr_type(field, ins, scope, return_type, generics, prefix)?;

				let assignee_type = assignee.typ;

				assignees.insert(name.clone(), assignee);

				let slot = if let Some(slot) = struct_members.get(&name) {
					*slot
				} else {
					return Err(InfoTypeError {
						span: expr.idx.clone(),
						error: TypeError::UnknownField(name.clone()),
					});
				};
				if !ins.compatible(assignee_type, slot, 0).unwrap() {
					return Err(InfoTypeError {
						span: expr.idx.clone(),
						error: TypeError::IncompatibleTypes {
							expected: ins.get_type(slot).cloned().unwrap(),
							got: ins.get_type(assignee_type).cloned().unwrap(),
						},
					});
				}
			}

			Ok(TypedExpr {
				typ: struct_type_id,
				expr: Expr::InitializeStruct(struct_type_id, assignees),
			})
		}
		Expr::Access(struct_expr, field_name) => {
			let struct_typed =
				infer_expr_type(*struct_expr, ins, scope, return_type, generics, prefix)?;

			let typ = if let Type::Concrete(ConcreteType::Struct(struct_type)) =
				ins.get_type(struct_typed.typ).unwrap()
			{
				if let Some(slot) = struct_type.get(&field_name) {
					Ok(*slot)
				} else {
					Err(InfoTypeError {
						span: expr.idx.clone(),
						error: TypeError::UnknownField(field_name.clone()),
					})
				}
			} else {
				Err(InfoTypeError {
					span: expr.idx.clone(),
					error: TypeError::NotAStruct(ins.get_type(struct_typed.typ).unwrap().clone()),
				})
			}?;

			Ok(TypedExpr {
				typ,
				expr: Expr::Access(Box::new(struct_typed), field_name.clone()),
			})
		}
		Expr::Is {
			name,
			typ: comparison_type,
		} => Ok(TypedExpr {
			typ: ins.add(Type::Concrete(ConcreteType::Bool)),
			expr: Expr::Is {
				name: name,
				typ: ins.instantiate(&comparison_type, generics)?,
			},
		}),
		Expr::Call(function_expr, args_exprs) => {
			let function_expr =
				infer_expr_type(*function_expr, ins, scope, return_type, generics, prefix)?;

			let (args, callee_return_type) =
				if let Type::Concrete(ConcreteType::Function(args, callee_return_type, _imp)) =
					ins.get_type(function_expr.typ).cloned().unwrap()
				{
					(args, callee_return_type)
				} else {
					return Err(InfoTypeError {
						span: expr.idx.clone(),
						error: TypeError::NotAFunction(
							ins.get_type(function_expr.typ).cloned().unwrap(),
						),
					});
				};

			if args_exprs.len() != args.len() {
				return Err(InfoTypeError {
					span: expr.idx.clone(),
					error: TypeError::IncorrectArgumentCount {
						expected: args.len(),
						got: args_exprs.len(),
					},
				});
			}

			let mut typed_arg_exprs = Vec::new();

			for i in 0..args.len() {
				let arg_expr = infer_expr_type(
					args_exprs[i].clone(),
					ins,
					scope,
					return_type,
					generics,
					prefix,
				)?;
				if !ins.compatible(arg_expr.typ, args[i], 0).unwrap() {
					return Err(InfoTypeError {
						span: expr.idx.clone(),
						error: TypeError::IncompatibleTypes {
							expected: ins.get_type(args[i]).cloned().unwrap(),
							got: ins.get_type(arg_expr.typ).cloned().unwrap(),
						},
					});
				}
				typed_arg_exprs.push(arg_expr);
			}

			Ok(TypedExpr {
				typ: callee_return_type,
				expr: Expr::Call(Box::new(function_expr), typed_arg_exprs),
			})
		}
		Expr::If { cond, then, els } => {
			let cond_typed = infer_expr_type(*cond, ins, scope, return_type, generics, prefix)?;
			let bool = ins.add(Type::Concrete(ConcreteType::Bool));
			if !ins.compatible(cond_typed.typ, bool, 0).unwrap() {
				return Err(InfoTypeError {
					span: expr.idx.clone(),
					error: TypeError::IncompatibleTypes {
						expected: Type::Concrete(ConcreteType::Bool),
						got: ins.get_type(cond_typed.typ).cloned().unwrap(),
					},
				});
			}
			let mut then_scope = scope.sub();
			if let Expr::Is { name, typ } = &cond_typed.expr {
				then_scope.insert(name.clone(), *typ);
			}
			let then_typed =
				infer_expr_type(*then, ins, &mut then_scope, return_type, generics, prefix)?;
			let els_typed = if let Some(els) = els {
				Some(infer_expr_type(
					*els,
					ins,
					scope,
					return_type,
					generics,
					prefix,
				)?)
			} else {
				None
			};
			let typ = if let Some(els_typed) = &els_typed {
				if !ins.compatible(then_typed.typ, els_typed.typ, 0).unwrap() {
					ins.add(Type::Union(then_typed.typ, els_typed.typ))
				} else {
					then_typed.typ
				}
			} else {
				then_typed.typ
			};

			Ok(TypedExpr {
				typ,
				expr: Expr::If {
					cond: Box::new(cond_typed),
					then: Box::new(then_typed),
					els: els_typed.map(Box::new),
				},
			})
		}
		Expr::Guard { dependency, body } => {
			let dependency =
				infer_expr_type(*dependency, ins, scope, return_type, generics, prefix)?;
			let body = infer_expr_type(*body, ins, scope, return_type, generics, prefix)?;
			Ok(TypedExpr {
				typ: body.typ,
				expr: Expr::Guard {
					dependency: Box::new(dependency),
					body: Box::new(body),
				},
			})
		}
		Expr::Index(_, _) => panic!("TODO: remove indexing until i add operator overloading"),
		Expr::Let(name, value_expr) => {
			let value_typed =
				infer_expr_type(*value_expr, ins, scope, return_type, generics, prefix)?;

			scope.insert(name.clone(), value_typed.typ);

			Ok(TypedExpr {
				typ: ins.add(Type::Concrete(ConcreteType::Tuple(Vec::new()))),
				expr: Expr::Let(name, Box::new(value_typed)),
			})
		}
		Expr::Return(return_expr) => {
			let (expr_type, out) = if let Some(expr) = return_expr {
				let return_expr_typed =
					infer_expr_type(*expr, ins, scope, return_type, generics, prefix)?;
				(
					return_expr_typed.typ,
					Expr::Return(Some(Box::new(return_expr_typed))),
				)
			} else {
				(
					ins.add(Type::Concrete(ConcreteType::Tuple(Vec::new()))),
					Expr::Return(None),
				)
			};

			if !ins.compatible(expr_type, return_type, 0).unwrap() {
				return Err(InfoTypeError {
					span: expr.idx.clone(),
					error: TypeError::IncompatibleTypes {
						expected: ins.get_type(return_type).cloned().unwrap(),
						got: ins.get_type(expr_type).cloned().unwrap(),
					},
				});
			}

			Ok(TypedExpr {
				typ: ins.add(Type::EarlyReturn),
				expr: out,
			})
		}
	}
}
