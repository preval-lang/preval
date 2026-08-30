use std::{borrow::Cow, collections::HashMap};

use crate::{
	parser::{expression::InfoExpr, typ::InfoTypeExpr},
	tokeniser::Literal,
	typ::{ConcreteType, Implementation, InfoTypeError, Instantiator, Type, TypeError, TypeExpr},
};

use crate::parser::expression::InfoExprKind;

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
	pub expr: TypedExprKind,
}

#[derive(Debug, Clone)]
pub enum TypedExprKind {
	Index(Box<TypedExpr>, Box<TypedExpr>),
	Name(usize),
	Literal(Literal),
	Call(Box<TypedExpr>, Vec<TypedExpr>),
	Return(Option<Box<TypedExpr>>),
	Block(Vec<TypedExpr>, bool),
	Let(String, Box<TypedExpr>),
	If {
		cond: Box<TypedExpr>,
		then: Box<TypedExpr>,
		els: Option<Box<TypedExpr>>,
	},
	InitializeStruct(usize, HashMap<String, TypedExpr>),
	Access(Box<TypedExpr>, String),
	Guard {
		dependency: Box<TypedExpr>,
		body: Box<TypedExpr>,
	},
	Is {
		name: String,
		typ: usize,
	},
	Local(String),
}

pub fn infer_expr_type<'a>(
	expr: InfoExpr<'a>,
	ins: &mut Instantiator<'a>,
	scope: &mut Scope,
	return_type: usize,
	generics: &[usize],
) -> Result<TypedExpr, InfoTypeError<'a>> {
	match expr.expr {
		InfoExprKind::Function(args, returns, body) => {
			let returns = returns
				.map(|r| ins.instantiate(&r, generics))
				.unwrap_or_else(|| Ok(ins.unification_var()))?;
			let mut arg_types = Vec::new();
			let mut arg_names = Vec::new();
			let mut scope = Scope::new();
			for arg in args {
				let typ = arg
					.1
					.map(|r| ins.instantiate(&r, generics))
					.unwrap_or_else(|| Ok(ins.unification_var()))?;
				arg_types.push(typ);

				arg_names.push(arg.0.clone());

				scope.insert(arg.0, typ);
			}

			let ir = ins.lower_function_body(
				body.as_ref(),
				&mut scope,
				returns,
				&expr.idx,
				&arg_names,
				generics,
			)?;

			let typ = ins.add(Type::Concrete(ConcreteType::Function(
				arg_types,
				returns,
				Implementation::Normal(ir),
			)));

			Ok(TypedExpr {
				typ,
				expr: TypedExprKind::Name(typ),
			})
		}
		InfoExprKind::MethodAccess(_, _, _) => {
			panic!("TODO: better error for using method as a value")
		}
		InfoExprKind::Literal(value) => Ok(TypedExpr {
			typ: ins.add(Type::Concrete(value.get_type())),
			expr: TypedExprKind::Literal(value.clone()),
		}),
		InfoExprKind::Name(name) => {
			if let InfoTypeExpr {
				expr: TypeExpr::Name(name, params),
				idx: _,
			} = &name
			{
				if params.len() == 0 {
					if let Some(type_id) = scope.get(name) {
						return Ok(TypedExpr {
							typ: type_id,
							expr: TypedExprKind::Local(name.clone()),
						});
					}
				}
			};
			let typ = ins.instantiate(&name, generics)?;
			Ok(TypedExpr {
				typ: typ.clone(),
				expr: TypedExprKind::Name(typ),
			})
		}
		InfoExprKind::Block(statements, returns) => {
			let mut scope = scope.sub();
			let mut typed_statements = Vec::new();
			for statement in statements {
				typed_statements.push(infer_expr_type(
					statement,
					ins,
					&mut scope,
					return_type,
					generics,
				)?);
			}

			let typ = if typed_statements.len() == 0 || !returns {
				ins.add(Type::Concrete(ConcreteType::Tuple(Vec::new())))
			} else {
				typed_statements.last().unwrap().typ
			};

			Ok(TypedExpr {
				typ,
				expr: TypedExprKind::Block(typed_statements, returns.clone()),
			})
		}
		InfoExprKind::InitializeStruct(struct_type_expr, fields) => {
			let struct_type_id = ins.instantiate(&struct_type_expr, generics)?;
			let struct_members =
				ins.expect_struct(struct_type_id)
					.map_err(|typ| InfoTypeError {
						span: struct_type_expr.idx.clone(),
						error: TypeError::NotAStruct(typ),
					})?;

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
				let assignee = infer_expr_type(field, ins, scope, return_type, generics)?;

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
				if !ins.compatible(assignee_type, slot) {
					return Err(InfoTypeError {
						span: expr.idx.clone(),
						error: TypeError::IncompatibleTypes {
							expected: ins.type_of(slot),
							got: ins.type_of(assignee_type),
						},
					});
				}
			}

			Ok(TypedExpr {
				typ: struct_type_id,
				expr: TypedExprKind::InitializeStruct(struct_type_id, assignees),
			})
		}
		InfoExprKind::Access(struct_expr, field_name) => {
			let struct_typed = infer_expr_type(*struct_expr, ins, scope, return_type, generics)?;

			let struct_members =
				ins.expect_struct(struct_typed.typ)
					.map_err(|typ| InfoTypeError {
						span: expr.idx.clone(),
						error: TypeError::NotAStruct(typ),
					})?;

			let typ = if let Some(slot) = struct_members.get(&field_name) {
				*slot
			} else {
				return Err(InfoTypeError {
					span: expr.idx.clone(),
					error: TypeError::UnknownField(field_name.clone()),
				});
			};

			Ok(TypedExpr {
				typ,
				expr: TypedExprKind::Access(Box::new(struct_typed), field_name.clone()),
			})
		}
		InfoExprKind::Is {
			name,
			typ: comparison_type,
		} => Ok(TypedExpr {
			typ: ins.add(Type::Concrete(ConcreteType::Bool)),
			expr: TypedExprKind::Is {
				name: name,
				typ: ins.instantiate(&comparison_type, generics)?,
			},
		}),
		InfoExprKind::Call(function_expr, mut args_exprs) => {
			let function_expr =
				if let InfoExprKind::MethodAccess(obj, method_name, params) = function_expr.expr {
					args_exprs = {
						let mut temp = vec![*obj.clone()];
						temp.extend(args_exprs.iter().cloned());
						temp
					};
					let obj = infer_expr_type(*obj, ins, scope, return_type, generics)?;

					let method = ins.instantiate_name(
						&method_name,
						&params,
						&expr.idx,
						generics,
						Some(obj.typ),
					)?;

					TypedExpr {
						typ: method,
						expr: TypedExprKind::Name(method),
					}
				} else {
					infer_expr_type(*function_expr, ins, scope, return_type, generics)?
				};

			let (args, callee_return_type) =
				ins.expect_function(function_expr.typ)
					.map_err(|typ| InfoTypeError {
						span: expr.idx.clone(),
						error: TypeError::NotAFunction(typ),
					})?;

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
				let arg_expr =
					infer_expr_type(args_exprs[i].clone(), ins, scope, return_type, generics)?;
				if !ins.compatible(arg_expr.typ, args[i]) {
					return Err(InfoTypeError {
						span: expr.idx.clone(),
						error: TypeError::IncompatibleTypes {
							expected: ins.type_of(args[i]),
							got: ins.type_of(arg_expr.typ),
						},
					});
				}
				typed_arg_exprs.push(arg_expr);
			}

			Ok(TypedExpr {
				typ: callee_return_type,
				expr: TypedExprKind::Call(Box::new(function_expr), typed_arg_exprs),
			})
		}
		InfoExprKind::If { cond, then, els } => {
			let cond_typed = infer_expr_type(*cond, ins, scope, return_type, generics)?;
			let bool = ins.add(Type::Concrete(ConcreteType::Bool));
			if !ins.compatible(cond_typed.typ, bool) {
				return Err(InfoTypeError {
					span: expr.idx.clone(),
					error: TypeError::IncompatibleTypes {
						expected: Type::Concrete(ConcreteType::Bool),
						got: ins.type_of(cond_typed.typ),
					},
				});
			}
			let mut then_scope = scope.sub();
			if let TypedExprKind::Is { name, typ } = &cond_typed.expr {
				then_scope.insert(name.clone(), *typ);
			}
			let then_typed = infer_expr_type(*then, ins, &mut then_scope, return_type, generics)?;
			let els_typed = if let Some(els) = els {
				Some(infer_expr_type(*els, ins, scope, return_type, generics)?)
			} else {
				None
			};
			let typ = if let Some(els_typed) = &els_typed {
				if !ins.compatible(then_typed.typ, els_typed.typ) {
					ins.add(Type::Union(then_typed.typ, els_typed.typ))
				} else {
					then_typed.typ
				}
			} else {
				then_typed.typ
			};

			Ok(TypedExpr {
				typ,
				expr: TypedExprKind::If {
					cond: Box::new(cond_typed),
					then: Box::new(then_typed),
					els: els_typed.map(Box::new),
				},
			})
		}
		InfoExprKind::Guard { dependency, body } => {
			let dependency = infer_expr_type(*dependency, ins, scope, return_type, generics)?;
			let body = infer_expr_type(*body, ins, scope, return_type, generics)?;
			Ok(TypedExpr {
				typ: body.typ,
				expr: TypedExprKind::Guard {
					dependency: Box::new(dependency),
					body: Box::new(body),
				},
			})
		}
		InfoExprKind::Index(_, _) => {
			panic!("TODO: remove indexing until i add operator overloading")
		}
		InfoExprKind::Let(name, value_expr) => {
			let value_typed = infer_expr_type(*value_expr, ins, scope, return_type, generics)?;

			scope.insert(name.clone(), value_typed.typ);

			Ok(TypedExpr {
				typ: ins.add(Type::Concrete(ConcreteType::Tuple(Vec::new()))),
				expr: TypedExprKind::Let(name, Box::new(value_typed)),
			})
		}
		InfoExprKind::Return(return_expr) => {
			let (expr_type, out) = if let Some(expr) = return_expr {
				let return_expr_typed = infer_expr_type(*expr, ins, scope, return_type, generics)?;
				(
					return_expr_typed.typ,
					TypedExprKind::Return(Some(Box::new(return_expr_typed))),
				)
			} else {
				(
					ins.add(Type::Concrete(ConcreteType::Tuple(Vec::new()))),
					TypedExprKind::Return(None),
				)
			};

			if !ins.compatible(expr_type, return_type) {
				return Err(InfoTypeError {
					span: expr.idx.clone(),
					error: TypeError::IncompatibleTypes {
						expected: ins.type_of(return_type),
						got: ins.type_of(expr_type),
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
