use std::collections::HashMap;

use crate::{
    expression_parser::{Expr, InfoExpr},
    tokeniser::Literal,
};

#[derive(Debug)]
pub struct IRErrorInfo {
    pub idx: usize,
    pub error: IRError,
}

#[derive(Debug)]
pub enum IRError {
    SymbolUndefined(String),
    SymbolNotCallable(String),
    SymbolNotIndexable(String),
    ExpressionNotCallable(Type),
    TypeMismatch { got: Type, expected: Type },
    ExtraArgument(),
}

#[derive(Debug)]

pub struct Module<VarRepr: Clone> {
    pub constants: Vec<(Type, Vec<u8>)>,
    pub functions: HashMap<String, Function<VarRepr>>,
}
#[derive(Debug)]
pub struct Function<VarRepr: Clone> {
    pub ir: Vec<Block<VarRepr>>,
    pub exported: bool,
    pub variable_types: Vec<Type>,
    pub signature: Signature,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    usize,
    void,
    Pointer(Pointer),
    u8,
    Slice(Box<Type>),
    Array(Box<Type>, usize),
    IO,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pointer {
    Function(Box<Signature>),
    Value(Box<Type>),
}

#[derive(Debug, Clone)]
pub enum Statement<VarRepr: Clone> {
    // maybe get rid of this, there aren't any non-operation statements after introducing Terminals and Blocks
    Operation(Operation<VarRepr>, Option<usize>),
}

#[derive(Debug, Clone)]
pub enum Operation<VarRepr: Clone> {
    Call {
        function: Vec<String>,
        args: Vec<usize>,
    },
    CallPointer {
        pointer: usize,
        args: Vec<usize>,
    },
    LoadGlobal {
        src: usize,
    },
    LoadLocal {
        src: usize,
    },
    PartialCall {
        function: Vec<Block<VarRepr>>,
        variables: HashMap<usize, Option<VarRepr>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Signature {
    pub(crate) args: Vec<Type>,
    pub(crate) returns: Type,
}

#[derive(Debug)]
pub enum Declaration {
    Module(HashMap<String, Declaration>),
    Function(Signature),
    Variable(usize),
}

#[derive(Debug, Clone)]
pub enum Terminal {
    Return(Option<usize>),
    Evaluate(Option<usize>),
}

#[derive(Debug, Clone)]
pub struct Block<VarRepr: Clone> {
    pub statements: Vec<Statement<VarRepr>>,
    pub terminal: Terminal,
}

pub fn to_ir<VarRepr: Clone>(
    function: &mut Function<VarRepr>,
    block: usize,
    module: &mut Module<VarRepr>,
    expr: InfoExpr,
    next_var: &mut usize,
    store: bool,
    declarations: &HashMap<String, Declaration>,
    locals: &mut HashMap<String, Declaration>,
    evaluate: bool,
) -> Result<(), IRErrorInfo> {
    let a = match expr.expr {
        Expr::Let(name, value) => {
            to_ir(
                function,
                block,
                module,
                *value,
                next_var,
                true,
                declarations,
                locals,
                false,
            )?;
            locals.insert(name, Declaration::Variable(*next_var));
            *next_var += 1;
            Ok(())
        }
        Expr::Return(expr) => {
            if let Some(expr) = expr {
                to_ir(
                    function,
                    block,
                    module,
                    *expr,
                    next_var,
                    true,
                    declarations,
                    locals,
                    false,
                )?;
                function.ir[block].terminal = Terminal::Return(Some(*next_var));

                *next_var += 1;
            } else {
                function.ir[block].terminal = Terminal::Return(Some(*next_var));
            }
            Ok(())
        }
        Expr::Block(statements, returns) => {
            let mut i = 0;
            let len = statements.len();
            for statement in statements {
                if i == len - 1 {
                    if store && returns {
                        to_ir(
                            function,
                            block,
                            module,
                            statement,
                            next_var,
                            true,
                            declarations,
                            locals,
                            false,
                        )?;
                    } else {
                        to_ir(
                            function,
                            block,
                            module,
                            statement,
                            next_var,
                            false,
                            declarations,
                            locals,
                            false,
                        )?;
                    }
                } else {
                    to_ir(
                        function,
                        block,
                        module,
                        statement,
                        next_var,
                        false,
                        declarations,
                        locals,
                        false,
                    )?;
                }
                i += 1;
            }
            Ok(())
        }
        Expr::Index(left, right) => todo!(),
        Expr::Var(name) => match locals.get(&name).or_else(|| declarations.get(&name)) {
            None => {
                return Err(IRErrorInfo {
                    idx: expr.idx,
                    error: IRError::SymbolUndefined(name),
                });
            }
            Some(Declaration::Variable(number)) => {
                if store {
                    let typ = function.variable_types[*number].clone();
                    function.variable_types.push(typ);
                    function.ir[block].statements.push(Statement::Operation(
                        Operation::LoadLocal { src: *number },
                        Some(*next_var),
                    ));
                }
                Ok(())
            }
            a => todo!("{a:?}"),
        },
        Expr::Literal(literal) => match literal {
            Literal::String(str) => {
                module
                    .constants
                    .push((Type::Array(Box::new(Type::u8), str.len()), str.into_bytes()));
                if store {
                    function.ir[block].statements.push(Statement::Operation(
                        Operation::LoadGlobal {
                            src: module.constants.len() - 1,
                        },
                        Some(*next_var),
                    ));
                    function
                        .variable_types
                        .push(Type::Slice(Box::new(Type::u8)));
                } else {
                    function.ir[block].statements.push(Statement::Operation(
                        Operation::LoadGlobal {
                            src: module.constants.len() - 1,
                        },
                        None,
                    ));
                }
                Ok(())
            }
            Literal::Number(num) => {
                module.constants.push((Type::u8, vec![num]));
                if store {
                    function.ir[block].statements.push(Statement::Operation(
                        Operation::LoadGlobal {
                            src: module.constants.len() - 1,
                        },
                        Some(*next_var),
                    ));
                    function.variable_types.push(Type::u8);
                } else {
                    function.ir[block].statements.push(Statement::Operation(
                        Operation::LoadGlobal {
                            src: module.constants.len() - 1,
                        },
                        None,
                    ));
                }
                Ok(())
            }
        },
        Expr::Call(callee, args) => {
            // TODO: check functions exist and have valid arguments
            let mut name = Vec::new();

            let callee = *callee;

            let mut arg_indexes = Vec::new();
            for arg in args {
                to_ir(
                    function,
                    block,
                    module,
                    arg,
                    next_var,
                    true,
                    declarations,
                    locals,
                    false,
                )?;
                arg_indexes.push(*next_var);
                *next_var += 1;
            }

            if mangle_name(&mut name, &callee) {
                let sig = match get_declaration(&name, &declarations, callee.idx)? {
                    Declaration::Function(sig) => sig,
                    _ => {
                        return Err(IRErrorInfo {
                            idx: callee.idx,
                            error: IRError::SymbolNotCallable(name.last().unwrap().to_string()),
                        });
                    }
                };
                for (arg_index, type_index) in arg_indexes.iter().enumerate() {
                    if let Some(typ) = sig.args.get(arg_index) {
                        if function.variable_types[*type_index] != *typ {
                            return Err(IRErrorInfo {
                                idx: callee.idx,
                                error: IRError::TypeMismatch {
                                    got: function.variable_types[*type_index].clone(),
                                    expected: sig.args[arg_index].clone(),
                                },
                            });
                        }
                    } else {
                        return Err(IRErrorInfo {
                            idx: expr.idx,
                            error: IRError::ExtraArgument(),
                        });
                    }
                }
                // TODO: use actual variable list to figure out function types
                if store {
                    function.variable_types.push(sig.returns.clone());
                    function.ir[block].statements.push(Statement::Operation(
                        Operation::Call {
                            function: name,
                            args: arg_indexes,
                        },
                        Some(*next_var),
                    ));
                } else {
                    function.ir[block].statements.push(Statement::Operation(
                        Operation::Call {
                            function: name,
                            args: arg_indexes,
                        },
                        None,
                    ));
                }
            } else {
                let function_idx = callee.idx;
                let function_var = *next_var;
                to_ir(
                    function,
                    block,
                    module,
                    callee,
                    next_var,
                    true,
                    declarations,
                    locals,
                    false,
                )?;
                *next_var += 1;
                let sig = match &function.variable_types[function_var] {
                    Type::Pointer(Pointer::Function(sig)) => sig,
                    ty => {
                        return Err(IRErrorInfo {
                            idx: function_idx,
                            error: IRError::ExpressionNotCallable(ty.clone()),
                        });
                    }
                };
                for (arg_index, type_index) in arg_indexes.iter().enumerate() {
                    if function.variable_types[*type_index] != sig.args[arg_index] {
                        return Err(IRErrorInfo {
                            idx: function_idx,
                            error: IRError::TypeMismatch {
                                got: function.variable_types[*type_index].clone(),
                                expected: sig.args[arg_index].clone(),
                            },
                        });
                    }
                }

                function.variable_types[*next_var] = sig.returns.clone();
                if store {
                    function.ir[block].statements.push(Statement::Operation(
                        Operation::CallPointer {
                            pointer: function_var,
                            args: arg_indexes,
                        },
                        Some(*next_var),
                    ));
                } else {
                    function.ir[block].statements.push(Statement::Operation(
                        Operation::CallPointer {
                            pointer: function_var,
                            args: arg_indexes,
                        },
                        None,
                    ));
                }
            }
            Ok(())
        }
    };
    if evaluate {
        function.ir[block].terminal =
            Terminal::Evaluate(if store { Some(*next_var) } else { None });
    }
    a
}

fn mangle_name(list: &mut Vec<String>, expr: &InfoExpr) -> bool {
    match &expr.expr {
        Expr::Var(name) => {
            list.push(name.to_string());
            true
        }
        Expr::Index(left, right) => {
            mangle_name(list, &*left);
            match &right.expr {
                Expr::Literal(Literal::String(name)) => {
                    list.push(name.to_string());
                    true
                }
                _ => false,
            }
        }
        _ => false,
    }
}

fn get_declaration<'a>(
    names: &[String],
    declarations: &'a HashMap<String, Declaration>,
    idx: usize,
) -> Result<&'a Declaration, IRErrorInfo> {
    if let Some(name) = names.get(0) {
        match declarations.get(name) {
            None => Err(IRErrorInfo {
                idx,
                error: IRError::SymbolUndefined(name.clone()),
            }),
            Some(decl) => {
                if names.len() > 1 {
                    match decl {
                        Declaration::Module(module) => get_declaration(&names[1..], module, idx),
                        _ => Err(IRErrorInfo {
                            idx,
                            error: IRError::SymbolNotIndexable(name.clone()),
                        }),
                    }
                } else {
                    Ok(decl)
                }
            }
        }
    } else {
        panic!("Incorrect input: Cannot get declaration with no name");
    }
}
