use std::{borrow::Cow, collections::HashMap};

use crate::{
	parser::{expression::InfoExpr, typ::InfoTypeExpr},
	typ::{ConcreteType, InfoTypeError, Program, Type, TypeError, TypeExpr},
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

pub fn infer_expr_type<'a>(
	expr: &InfoExpr<'a>,
	ins: &mut Program<'a>,
	scope: &mut Scope,
	return_type: usize,
	generics: &[usize],
	prefix: &[String],
) -> Result<usize, InfoTypeError<'a>> {
	match &expr.expr {
		Expr::Literal(value) => Ok(ins.add(Type::Concrete(value.get_type()))),
		Expr::Name(name) => {
			if let InfoTypeExpr {
				expr: TypeExpr::Name(name, false),
				idx: _,
			} = name
			{
				if name.len() == 1 {
					if let Some(type_id) = scope.get(&name[0]) {
						return Ok(type_id);
					}
				}
			};
			ins.instantiate(name, generics, prefix)
		}
		Expr::Block(statements, returns) => {
			if statements.len() == 0 || (statements.len() == 1 && !returns) {
				return Ok(ins.add(Type::Concrete(ConcreteType::Tuple(Vec::new()))));
			}

			let mut scope = scope.sub();
			for statement in &statements[..statements.len() - 1] {
				let _ = infer_expr_type(statement, ins, &mut scope, return_type, generics, prefix)?;
			}
			if *returns {
				infer_expr_type(
					statements.last().unwrap(),
					ins,
					&mut scope,
					return_type,
					generics,
					prefix,
				)
			} else {
				Ok(ins.add(Type::Concrete(ConcreteType::Tuple(Vec::new()))))
			}
		}
		Expr::InitializeStruct(struct_type_expr, fields) => {
			let struct_type_id = ins.instantiate(struct_type_expr, generics, prefix)?;
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

			for (name, field) in fields {
				let assignee_type =
					infer_expr_type(field, ins, scope, return_type, generics, prefix)?;

				let slot = if let Some(slot) = struct_members.get(name) {
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

			Ok(struct_type_id)
		}
		Expr::Access(struct_expr, field_name) => {
			let struct_type_id =
				infer_expr_type(struct_expr, ins, scope, return_type, generics, prefix)?;

			if let Type::Concrete(ConcreteType::Struct(struct_type)) =
				ins.get_type(struct_type_id).unwrap()
			{
				if let Some(slot) = struct_type.get(field_name) {
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
					error: TypeError::NotAStruct(ins.get_type(struct_type_id).unwrap().clone()),
				})
			}
		}
		Expr::Is { name: _, typ: _ } => Ok(ins.add(Type::Concrete(ConcreteType::Bool))),
		Expr::Call(function_expr, args_exprs) => {
			let function_type_id =
				infer_expr_type(function_expr, ins, scope, return_type, generics, prefix)?;

			let (args, callee_return_type) =
				if let Type::Concrete(ConcreteType::Function(args, callee_return_type, _imp)) =
					ins.get_type(function_type_id).cloned().unwrap()
				{
					(args, callee_return_type)
				} else {
					return Err(InfoTypeError {
						span: expr.idx.clone(),
						error: TypeError::NotAFunction(
							ins.get_type(function_type_id).cloned().unwrap(),
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

			for (arg_expr, arg_type) in args_exprs.iter().zip(args.iter()) {
				let arg_expr_type =
					infer_expr_type(arg_expr, ins, scope, return_type, generics, prefix)?;
				if !ins.compatible(arg_expr_type, *arg_type, 0).unwrap() {
					return Err(InfoTypeError {
						span: expr.idx.clone(),
						error: TypeError::IncompatibleTypes {
							expected: ins.get_type(*arg_type).cloned().unwrap(),
							got: ins.get_type(arg_expr_type).cloned().unwrap(),
						},
					});
				}
			}

			Ok(callee_return_type)
		}
		Expr::If { cond, then, els } => {
			let cond_type = infer_expr_type(cond, ins, scope, return_type, generics, prefix)?;
			let bool = ins.add(Type::Concrete(ConcreteType::Bool));
			if !ins.compatible(cond_type, bool, 0).unwrap() {
				return Err(InfoTypeError {
					span: expr.idx.clone(),
					error: TypeError::IncompatibleTypes {
						expected: Type::Concrete(ConcreteType::Bool),
						got: ins.get_type(cond_type).cloned().unwrap(),
					},
				});
			}
			let mut then_scope = scope.sub();
			if let Expr::Is { name, typ } = &cond.expr {
				let typ = ins.instantiate(typ, generics, prefix)?;
				then_scope.insert(name.clone(), typ);
			}
			let then_type =
				infer_expr_type(then, ins, &mut then_scope, return_type, generics, prefix)?;
			let els_type = if let Some(els) = els {
				Some(infer_expr_type(
					els,
					ins,
					scope,
					return_type,
					generics,
					prefix,
				)?)
			} else {
				None
			};
			Ok(if let Some(els_type) = els_type {
				if !ins.compatible(then_type, els_type, 0).unwrap() {
					ins.add(Type::Union(then_type, els_type))
				} else {
					then_type
				}
			} else {
				then_type
			})
		}
		Expr::Guard { dependency, body } => {
			let _ = infer_expr_type(dependency, ins, scope, return_type, generics, prefix)?;
			let body_type = infer_expr_type(body, ins, scope, return_type, generics, prefix)?;
			Ok(body_type)
		}
		Expr::Index(_, _) => panic!("TODO: remove indexing until i add operator overloading"),
		Expr::Let(name, value_expr) => {
			let value_type_id =
				infer_expr_type(value_expr, ins, scope, return_type, generics, prefix)?;

			scope.insert(name.clone(), value_type_id);

			Ok(ins.add(Type::Concrete(ConcreteType::Tuple(Vec::new()))))
		}
		Expr::Return(return_expr) => {
			let expr_type = if let Some(expr) = return_expr {
				infer_expr_type(expr, ins, scope, return_type, generics, prefix)?
			} else {
				ins.add(Type::Concrete(ConcreteType::Tuple(Vec::new())))
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

			Ok(ins.add(Type::EarlyReturn))
		}
	}
}
