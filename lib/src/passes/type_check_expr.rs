use std::{borrow::Cow, cell::RefCell, collections::HashMap, sync::RwLock};

use crate::{
    parser::expression::{Expr, InfoExpr},
    typ::{ConcreteType, InfoTypeError, Instantiator, Type, TypeError},
};

#[derive(Debug)]
pub struct Scope<'a> {
    scopes: Vec<Cow<'a, HashMap<String, usize>>>,
}

impl<'a> Scope<'a> {
    pub fn new(global_scope: HashMap<String, usize>) -> Self {
        Self {
            scopes: vec![Cow::Owned(global_scope)],
        }
    }

    pub fn sub(&'a self) -> Scope<'a> {
        let mut scopes = self.scopes.clone();
        scopes.push(Cow::Owned(HashMap::new()));
        Self { scopes }
    }

    pub fn get(&self, name: &str) -> Option<usize> {
        for scope in &self.scopes {
            if let Some(typ) = (*scope).get(name) {
                return Some(*typ);
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

pub fn infer_expr_type(
    expr: &InfoExpr,
    ins: &mut Instantiator,
    scope: &mut Scope,
    return_type: usize,
) -> Result<usize, InfoTypeError> {
    match &expr.expr {
        Expr::Literal(value) => Ok(ins.concrete(value.get_type())),
        Expr::Var(name) => {
            if let Some(idx) = scope.get(name) {
                Ok(idx)
            } else {
                Err(InfoTypeError {
                    idx: expr.idx,
                    error: TypeError::UnknownVariable(name.clone()),
                })
            }
        }
        Expr::Block(statements, returns) => {
            if statements.len() == 0 || (statements.len() == 1 && !returns) {
                return Ok(ins.concrete(ConcreteType::Tuple(Vec::new())));
            }

            let mut scope = scope.sub();
            for statement in &statements[..statements.len() - 1] {
                let _ = infer_expr_type(statement, ins, &mut scope, return_type)?;
            }
            if *returns {
                infer_expr_type(statements.last().unwrap(), ins, &mut scope, return_type)
            } else {
                Ok(ins.concrete(ConcreteType::Tuple(Vec::new())))
            }
        }
        Expr::InitializeStruct(name, fields) => {
            let (struct_type, struct_type_id) = if let Some(struct_type_id) = ins.get_name(name) {
                if let Type::Concrete(ConcreteType::Struct(struct_type)) =
                    ins.get_type(struct_type_id).clone()
                {
                    (struct_type, struct_type_id)
                } else {
                    return Err(InfoTypeError {
                        idx: expr.idx,
                        error: TypeError::NotAStruct(ins.get_type(struct_type_id).clone()),
                    });
                }
            } else {
                return Err(InfoTypeError {
                    idx: expr.idx,
                    error: TypeError::UnknownType(name.clone()),
                });
            };

            if fields.len() != struct_type.len() {
                return Err(InfoTypeError {
                    idx: expr.idx,
                    error: TypeError::IncorrectFieldCount {
                        expected: struct_type.len(),
                        got: fields.len(),
                    },
                });
            }

            for (name, field) in fields {
                let assignee_type = infer_expr_type(field, ins, scope, return_type)?;

                let slot = if let Some(slot) = struct_type.get(name) {
                    *slot
                } else {
                    return Err(InfoTypeError {
                        idx: expr.idx,
                        error: TypeError::UnknownField(name.clone()),
                    });
                };
                if !ins.compatible(assignee_type, slot) {
                    return Err(InfoTypeError {
                        idx: expr.idx,
                        error: TypeError::IncompatibleTypes {
                            expected: ins.get_type(slot).clone(),
                            got: ins.get_type(assignee_type).clone(),
                        },
                    });
                }
            }

            Ok(struct_type_id)
        }
        Expr::Access(struct_expr, field_name) => {
            let struct_type_id = infer_expr_type(struct_expr, ins, scope, return_type)?;

            if let Type::Concrete(ConcreteType::Struct(struct_type)) = ins.get_type(struct_type_id)
            {
                if let Some(slot) = struct_type.get(field_name) {
                    Ok(*slot)
                } else {
                    Err(InfoTypeError {
                        idx: expr.idx,
                        error: TypeError::UnknownField(field_name.clone()),
                    })
                }
            } else {
                Err(InfoTypeError {
                    idx: expr.idx,
                    error: TypeError::UnknownType(field_name.clone()),
                })
            }
        }
        Expr::Call(function_expr, args_exprs) => {
            let function_type_id = infer_expr_type(function_expr, ins, scope, return_type)?;

            let signature = if let Type::Concrete(ConcreteType::Function(signature)) =
                ins.get_type(function_type_id).clone()
            {
                signature
            } else {
                return Err(InfoTypeError {
                    idx: expr.idx,
                    error: TypeError::NotAFunction(ins.get_type(function_type_id).clone()),
                });
            };

            if args_exprs.len() != signature.args.len() {
                return Err(InfoTypeError {
                    idx: expr.idx,
                    error: TypeError::IncorrectArgumentCount {
                        expected: signature.args.len(),
                        got: args_exprs.len(),
                    },
                });
            }

            for (arg_expr, arg_type) in args_exprs.iter().zip(signature.args.iter()) {
                let arg_expr_type = infer_expr_type(arg_expr, ins, scope, return_type)?;
                if !ins.compatible(arg_expr_type, *arg_type) {
                    return Err(InfoTypeError {
                        idx: expr.idx,
                        error: TypeError::IncompatibleTypes {
                            expected: ins.get_type(*arg_type).clone(),
                            got: ins.get_type(arg_expr_type).clone(),
                        },
                    });
                }
            }

            Ok(signature.returns)
        }
        Expr::If { cond, then, els } => {
            let cond_type = infer_expr_type(cond, ins, scope, return_type)?;
            let bool = ins.concrete(ConcreteType::Bool);
            if !ins.compatible(cond_type, bool) {
                return Err(InfoTypeError {
                    idx: expr.idx,
                    error: TypeError::IncompatibleTypes {
                        expected: Type::Concrete(ConcreteType::Bool),
                        got: ins.get_type(cond_type).clone(),
                    },
                });
            }
            let then_type = infer_expr_type(then, ins, scope, return_type)?;
            let els_type = if let Some(els) = els {
                Some(infer_expr_type(els, ins, scope, return_type)?)
            } else {
                None
            };
            Ok(if let Some(els_type) = els_type {
                if !ins.compatible(then_type, els_type) {
                    ins.add(Type::Union(then_type, els_type))
                } else {
                    then_type
                }
            } else {
                then_type
            })
        }
        Expr::Guard { dependency, body } => {
            let _ = infer_expr_type(dependency, ins, scope, return_type)?;
            let body_type = infer_expr_type(body, ins, scope, return_type)?;
            Ok(body_type)
        }
        Expr::Index(_, _) => panic!("TODO: remove indexing until i add operator overloading"),
        Expr::Let(name, value_expr) => {
            let value_type = infer_expr_type(value_expr, ins, scope, return_type)?;
            scope.insert(name.clone(), value_type);

            Ok(ins.add(Type::Concrete(ConcreteType::Tuple(Vec::new()))))
        }
        Expr::Return(return_expr) => {
            let expr_type = if let Some(expr) = return_expr {
                infer_expr_type(expr, ins, scope, return_type)?
            } else {
                ins.add(Type::Concrete(ConcreteType::Tuple(Vec::new())))
            };

            if !ins.compatible(expr_type, return_type) {
                return Err(InfoTypeError {
                    idx: expr.idx,
                    error: TypeError::IncompatibleTypes {
                        expected: ins.get_type(return_type).clone(),
                        got: ins.get_type(expr_type).clone(),
                    },
                });
            }

            Ok(ins.add(Type::EarlyReturn))
        }
    }
}
