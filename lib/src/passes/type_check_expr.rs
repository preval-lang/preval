use std::{borrow::Cow, cell::RefCell, collections::HashMap, sync::RwLock};

use crate::{
    ir::Module,
    parser::expression::{Expr, InfoExpr},
    typ::{
        Type,
        error::{InfoTypeError, TypeError},
        type_names,
    },
};

#[derive(Debug)]
pub struct Scope<'a> {
    scopes: Vec<Cow<'a, HashMap<String, Type>>>,
}

impl<'a> Scope<'a> {
    pub fn new(global_scope: HashMap<String, Type>) -> Self {
        Self {
            scopes: vec![Cow::Owned(global_scope)],
        }
    }

    pub fn sub(&'a self) -> Scope<'a> {
        let mut scopes = self.scopes.clone();
        scopes.push(Cow::Owned(HashMap::new()));
        Self { scopes }
    }

    pub fn get(&self, name: &str) -> Option<Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(typ) = (*scope).get(name) {
                return Some(typ.clone());
            }
        }
        None
    }

    pub fn insert(&mut self, name: String, typ: Type) {
        let mut last = self.scopes.pop().unwrap().into_owned();
        last.insert(name, typ);
        self.scopes.push(Cow::Owned(last));
    }
}

pub fn infer_expr_type(
    expr: &InfoExpr,
    module: &Module,
    scope: &mut Scope,
    return_type: Type,
) -> Result<Type, InfoTypeError> {
    match &expr.expr {
        Expr::Literal(value) => Ok(value.get_type()),
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
                return Ok(Type::Tuple(Vec::new()));
            }

            let mut scope = scope.sub();
            for statement in &statements[..statements.len() - 1] {
                let _ = infer_expr_type(statement, module, &mut scope, return_type.clone())?;
            }
            if *returns {
                infer_expr_type(statements.last().unwrap(), module, &mut scope, return_type)
            } else {
                Ok(Type::Tuple(Vec::new()))
            }
        }
        Expr::InitializeStruct(struct_type, fields) => {
            let def_fields =
                if let Type::Struct(def_fields, _) = resolve(struct_type, module, expr.idx)? {
                    def_fields
                } else {
                    return Err(InfoTypeError {
                        idx: expr.idx,
                        error: TypeError::NotAStruct(struct_type.clone()),
                    });
                };

            if fields.len() != def_fields.len() {
                return Err(InfoTypeError {
                    idx: expr.idx,
                    error: TypeError::IncorrectFieldCount {
                        expected: def_fields.len(),
                        got: fields.len(),
                    },
                });
            }

            for (name, field) in fields {
                let assignee_type = infer_expr_type(field, module, scope, return_type.clone())?;

                let slot = if let Some(slot) = def_fields.get(name) {
                    slot.clone()
                } else {
                    return Err(InfoTypeError {
                        idx: expr.idx,
                        error: TypeError::UnknownField(name.clone()),
                    });
                };
                if !compatible(&assignee_type, &slot, module, false)? {
                    return Err(InfoTypeError {
                        idx: expr.idx,
                        error: TypeError::IncompatibleTypes {
                            expected: slot.clone(),
                            got: assignee_type.clone(),
                        },
                    });
                }
            }

            Ok(struct_type.clone())
        }
        Expr::Access(struct_expr, field_name) => {
            let struct_type = infer_expr_type(struct_expr, module, scope, return_type)?;

            if let Type::Struct(fields, _) = resolve(&struct_type, module, expr.idx)? {
                if let Some(slot) = fields.get(field_name) {
                    Ok(slot.clone())
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
        Expr::Is { name: _, typ: _ } => Ok(type_names::bool()),
        Expr::Call(function_expr, args_exprs) => {
            let function_type = infer_expr_type(function_expr, module, scope, return_type.clone())?;

            let signature = if let Type::Function(signature) = function_type {
                signature
            } else {
                return Err(InfoTypeError {
                    idx: expr.idx,
                    error: TypeError::NotAFunction(function_type),
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
                let arg_expr_type = infer_expr_type(arg_expr, module, scope, return_type.clone())?;
                if !compatible(&arg_expr_type, arg_type, module, false)? {
                    return Err(InfoTypeError {
                        idx: expr.idx,
                        error: TypeError::IncompatibleTypes {
                            expected: arg_type.clone(),
                            got: arg_expr_type,
                        },
                    });
                }
            }

            Ok(*signature.returns)
        }
        Expr::If { cond, then, els } => {
            let cond_type = infer_expr_type(cond, module, scope, return_type.clone())?;
            if !compatible(&cond_type, &type_names::bool(), module, false)? {
                return Err(InfoTypeError {
                    idx: expr.idx,
                    error: TypeError::IncompatibleTypes {
                        expected: type_names::bool(),
                        got: cond_type,
                    },
                });
            }
            let mut then_scope = scope.sub();
            if let Expr::Is { name, typ } = &cond.expr {
                then_scope.insert(name.clone(), typ.clone());
            }
            let then_type = infer_expr_type(then, module, &mut then_scope, return_type.clone())?;
            let els_type = if let Some(els) = els {
                Some(infer_expr_type(els, module, scope, return_type.clone())?)
            } else {
                None
            };
            Ok(if let Some(els_type) = els_type {
                if !compatible(&then_type, &els_type, module, false)? {
                    Type::Union(Box::new(then_type), Box::new(els_type))
                } else {
                    then_type
                }
            } else {
                then_type
            })
        }
        Expr::Guard { dependency, body } => {
            let _ = infer_expr_type(dependency, module, scope, return_type.clone())?;
            let body_type = infer_expr_type(body, module, scope, return_type)?;
            Ok(body_type)
        }
        Expr::Index(_, _) => panic!("TODO: remove indexing until i add operator overloading"),
        Expr::Let(name, value_expr) => {
            let value_type = infer_expr_type(value_expr, module, scope, return_type)?;
            scope.insert(name.clone(), value_type);

            Ok(Type::Tuple(Vec::new()))
        }
        Expr::Return(return_expr) => {
            let expr_type = if let Some(expr) = return_expr {
                infer_expr_type(expr, module, scope, return_type.clone())?
            } else {
                Type::Tuple(Vec::new())
            };

            if !compatible(&expr_type, &return_type, module, false)? {
                return Err(InfoTypeError {
                    idx: expr.idx,
                    error: TypeError::IncompatibleTypes {
                        expected: return_type,
                        got: expr_type,
                    },
                });
            }

            Ok(Type::EarlyReturn)
        }
    }
}

fn resolve(typ: &Type, module: &Module, idx: usize) -> Result<Type, InfoTypeError> {
    if let Type::Named(name) = typ {
        if let Some(typ) = module.types.get(&name.path[0]) {
            return resolve(&typ, module, idx);
        } else {
            return Err(InfoTypeError {
                idx,
                error: TypeError::UnknownType(name.path[0].clone()),
            });
        }
    } else {
        return Ok(typ.clone());
    }
}

pub fn compatible(
    t1: &Type,
    t2: &Type,
    module: &Module,
    resolved: bool,
) -> Result<bool, InfoTypeError> {
    Ok(match t2 {
        Type::Union(t2a, t2b) => {
            compatible(
                &resolve(t2a, module, 18298)?,
                &resolve(t1, module, 18298)?,
                module,
                true,
            )? || compatible(
                &resolve(t2b, module, 18298)?,
                &resolve(t1, module, 18298)?,
                module,
                true,
            )?
        }
        Type::EarlyReturn => false,
        _ => {
            t1 == t2
                || (if !resolved {
                    compatible(
                        &resolve(t1, module, 18298)?,
                        &resolve(t2, module, 18298)?,
                        module,
                        true,
                    )?
                } else {
                    false
                })
        }
    })
}
