use std::{any::type_name_of_val, collections::HashMap};

use serde::{Deserialize, Serialize};

use crate::{
    expression_parser::{Expr, InfoExpr},
    tokeniser::Literal,
    typ::{Pointer, Signature, Type},
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
    NotStorable(String),
    MissingElseBlock(),
}

#[derive(Debug, Serialize, Deserialize)]

pub struct Module {
    pub constants: Vec<Vec<u8>>,
    pub functions: HashMap<String, Function>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub ir: Vec<Block>,
    pub exported: bool,
    pub variable_types: HashMap<usize, Type>,
    pub signature: Signature,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Statement {
    // maybe get rid of this, there aren't any non-operation statements after introducing Terminals and Blocks
    Operation(Operation, Option<usize>),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Operation {
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
    Phi {
        block_to_var: HashMap<usize, usize>,
    },
}

#[derive(Debug)]
pub enum Declaration {
    Module(HashMap<String, Declaration>),
    Function(Signature),
    Variable(usize),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Terminal {
    Return(Option<usize>),
    Evaluate(Option<usize>),
    Jump(usize),
    CondJump {
        cond: usize,
        then: usize,
        els: usize,
    },
    ThenElseJump(usize),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub terminal: Terminal,
}

pub fn to_ir(
    function: &mut Function,
    block: &mut usize,
    module: &mut Module,
    expr: InfoExpr,
    store: Option<usize>,
    declarations: &HashMap<String, Declaration>,
    locals: &mut HashMap<String, Declaration>,
) -> Result<(), IRErrorInfo> {
    match expr.expr {
        Expr::Literal(lit) => {
            if let Some(store) = store {
                let (typ, value) = match lit {
                    Literal::String(str) => (Type::Slice(Box::new(Type::u8)), str.into_bytes()),
                    Literal::Number(n) => (Type::u8, vec![n]),
                };
                function.variable_types.insert(store, typ.clone());
                function.ir[*block].statements.push(Statement::Operation(
                    Operation::LoadGlobal {
                        src: module.constants.len(),
                    },
                    Some(store),
                ));
                module.constants.push(value);
            }
            Ok(())
        }
        Expr::Let(name, value_expr) => {
            let value_var = function.variable_types.len();
            to_ir(
                function,
                block,
                module,
                *value_expr,
                Some(value_var),
                declarations,
                locals,
            )?;
            locals.insert(name, Declaration::Variable(value_var));
            if let Some(store) = store {
                function.ir[*block].statements.push(Statement::Operation(
                    Operation::LoadLocal { src: value_var },
                    Some(store),
                ));
            }
            Ok(())
        }
        Expr::Block(statements, returns) => {
            let mut i = 0;
            let len = statements.len();
            for statement in statements {
                if i != len - 1 || !returns {
                    to_ir(
                        function,
                        block,
                        module,
                        statement,
                        None,
                        declarations,
                        locals,
                    )?;
                } else {
                    to_ir(
                        function,
                        block,
                        module,
                        statement,
                        store,
                        declarations,
                        locals,
                    )?;
                }
                i += 1;
            }

            if (len == 0 || !returns) && store.is_some() {
                function.ir[*block].statements.push(Statement::Operation(
                    Operation::LoadGlobal {
                        src: module.constants.len(),
                    },
                    store,
                ));
                module.constants.push(Vec::new());
                function
                    .variable_types
                    .insert(store.unwrap(), Type::Tuple(Vec::new()));
            }
            Ok(())
        }
        Expr::Return(value_expr) => {
            function.ir[*block].terminal = Terminal::Return(if let Some(value_expr) = value_expr {
                let return_var = function.variable_types.len();
                to_ir(
                    function,
                    block,
                    module,
                    *value_expr,
                    Some(return_var),
                    declarations,
                    locals,
                )?;
                Some(return_var)
            } else {
                None
            });
            Ok(())
        }
        Expr::Call(callee, args) => {
            let mut name = Vec::new();

            let callee = *callee;

            let mut arg_indexes = Vec::new();
            for arg in args {
                let i = function.variable_types.len();
                to_ir(function, block, module, arg, Some(i), declarations, locals)?;
                arg_indexes.push(i);
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
                        if function.variable_types[type_index] != *typ {
                            return Err(IRErrorInfo {
                                idx: callee.idx,
                                error: IRError::TypeMismatch {
                                    got: function.variable_types[type_index].clone(),
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
                if let Some(store) = store {
                    function.variable_types.insert(store, sig.returns.clone());
                    function.ir[*block].statements.push(Statement::Operation(
                        Operation::Call {
                            function: name,
                            args: arg_indexes,
                        },
                        Some(store),
                    ));
                } else {
                    function.ir[*block].statements.push(Statement::Operation(
                        Operation::Call {
                            function: name,
                            args: arg_indexes,
                        },
                        None,
                    ));
                }
            }
            Ok(())
        }
        Expr::Var(name) => {
            if let Some(store) = store {
                function.ir[*block].statements.push(Statement::Operation(
                    Operation::LoadLocal {
                        src: match locals.get(&name).or(declarations.get(&name)) {
                            None => {
                                return Err(IRErrorInfo {
                                    idx: expr.idx,
                                    error: IRError::SymbolUndefined(name),
                                });
                            }
                            Some(Declaration::Variable(v)) => {
                                function
                                    .variable_types
                                    .insert(store, function.variable_types[v].clone());
                                *v
                            }
                            _ => {
                                return Err(IRErrorInfo {
                                    idx: expr.idx,
                                    error: IRError::NotStorable(name),
                                });
                            }
                        },
                    },
                    Some(store),
                ));
            }
            Ok(())
        }
        Expr::If { cond, then, els } => {
            // println!("IF: {store:?}");

            let cond_var = function.variable_types.len();
            to_ir(
                function,
                block,
                module,
                *cond,
                Some(cond_var),
                declarations,
                locals,
            )?;

            let then_block_n = function.ir.len();
            let mut then_block_n_mut = function.ir.len();
            let then_block_var = function.variable_types.len();
            function.ir.push(Block {
                statements: Vec::new(),
                terminal: Terminal::Jump(function.ir.len() + 1 + if els.is_some() { 1 } else { 0 }),
            });
            to_ir(
                function,
                &mut then_block_n_mut,
                module,
                *then,
                Some(then_block_var),
                declarations,
                locals,
            )?;

            let else_block = if let Some(els) = els {
                let else_block_n = function.ir.len();
                let mut else_block_n_mut = function.ir.len();
                let else_block_var = function.variable_types.len();
                function.ir.push(Block {
                    statements: Vec::new(),
                    terminal: Terminal::Jump(function.ir.len() + 1),
                });
                to_ir(
                    function,
                    &mut else_block_n_mut,
                    module,
                    *els,
                    Some(else_block_var),
                    declarations,
                    locals,
                )?;
                Some((else_block_n, else_block_var))
            } else {
                None
            };

            let old_terminal = function.ir[*block].terminal.clone();

            function.ir[*block].terminal = Terminal::CondJump {
                cond: cond_var,
                then: then_block_n,
                els: else_block.map(|f| f.0).unwrap_or(function.ir.len()),
            };
            *block = function.ir.len();

            function.ir.push(Block {
                statements: Vec::new(),
                terminal: old_terminal,
            });

            if let Some(store) = store {
                if let Some(else_block) = else_block {
                    let mut block_to_var = HashMap::new();
                    block_to_var.insert(else_block.0, else_block.1);
                    block_to_var.insert(then_block_n, then_block_var);
                    function.ir[*block].statements.push(Statement::Operation(
                        Operation::Phi { block_to_var },
                        Some(store),
                    ));
                    function
                        .variable_types
                        .insert(store, function.variable_types[&then_block_var].clone());
                } else {
                    return Err(IRErrorInfo {
                        idx: expr.idx,
                        error: IRError::MissingElseBlock(),
                    });
                }
            }

            Ok(())
        }
        _ => panic!("other exprs"),
    }
}

// pub fn to_ir<VarRepr: PartialEq+Clone>(
//     function: &mut Function,
//     block: &mut usize,
//     module: &mut Module,
//     expr: InfoExpr,
//     next_var: &mut usize,
//     store: bool,
//     declarations: &HashMap<String, Declaration>,
//     locals: &mut HashMap<String, Declaration>,
//     evaluate: bool,
// ) -> Result<Result<(), EarlyReturn>, IRErrorInfo> {
//     let a = match expr.expr {
//         Expr::If { cond, then, els } => {
//             if to_ir(
//                 function,
//                 block,
//                 module,
//                 *cond,
//                 next_var,
//                 true,
//                 declarations,
//                 locals,
//                 false,
//             )? {
//                 return Ok(true);
//             }

//             let cont_block_n = function.ir.len();
//             function.ir.push(Block {
//                 statements: Vec::new(),
//                 terminal: Terminal::Evaluate(None),
//             });

//             let mut then_block_n = function.ir.len();
//             function.ir.push(Block {
//                 statements: Vec::new(),
//                 terminal: Terminal::Jump(cont_block_n),
//             });

//             let else_block_n = if let Some(els) = &els {
//                 function.ir.push(Block {
//                     statements: Vec::new(),
//                     terminal: Terminal::Jump(cont_block_n),
//                 });
//                 Some(function.ir.len() - 1)
//             } else {
//                 None
//             };

//             function.ir[*block].terminal = Terminal::CondJump {
//                 cond: *next_var,
//                 then: then_block_n,
//                 els: else_block_n.unwrap_or(cont_block_n),
//             };

//             *next_var += 1;

//             to_ir(
//                 function,
//                 &mut then_block_n,
//                 module,
//                 *then,
//                 next_var,
//                 store,
//                 declarations,
//                 locals,
//                 true,
//             )?;

//             let then_var = *next_var;

//             *next_var += 1;

//             *block = cont_block_n;

//             if let Some(els) = els {
//                 to_ir(
//                     function,
//                     &mut else_block_n.unwrap(),
//                     module,
//                     *els,
//                     next_var,
//                     store,
//                     declarations,
//                     locals,
//                     true,
//                 )?;

//                 if store {
//                     let mut phi = HashMap::new();
//                     phi.insert(then_block_n, then_var);
//                     phi.insert(else_block_n.unwrap(), *next_var);
//                     function.ir[*block].statements.push(Statement::Operation(
//                         Operation::Phi { block_to_var: phi },
//                         Some(*next_var + 1),
//                     ));
//                     function
//                         .variable_types
//                         .push(function.variable_types[then_var].clone());
//                 }
//                 *next_var += 1;
//             }

//             Ok(false)
//         }
//         Expr::Let(name, value) => {
//             if to_ir(
//                 function,
//                 block,
//                 module,
//                 *value,
//                 next_var,
//                 true,
//                 declarations,
//                 locals,
//                 false,
//             )? {
//                 return Ok(true);
//             }
//             locals.insert(name, Declaration::Variable(*next_var));
//             *next_var += 1;
//             Ok(false)
//         }
//         Expr::Return(expr) => {
//             if let Some(expr) = expr {
//                 if to_ir(
//                     function,
//                     block,
//                     module,
//                     *expr,
//                     next_var,
//                     true,
//                     declarations,
//                     locals,
//                     false,
//                 )? {
//                     return Ok(true);
//                 }
//                 function.ir[*block].terminal = Terminal::Return(Some(*next_var));

//                 *next_var += 1;
//             } else {
//                 function.ir[*block].terminal = Terminal::Return(None);
//             }
//             Ok(true)
//         }
//         Expr::Block(statements, returns) => {
//             let mut i = 0;
//             let len = statements.len();
//             for statement in statements {
//                 if i == len - 1 {
//                     if store && returns {
//                         if to_ir(
//                             function,
//                             block,
//                             module,
//                             statement,
//                             next_var,
//                             true,
//                             declarations,
//                             locals,
//                             false,
//                         )? {
//                             return Ok(true);
//                         }
//                     } else {
//                         if to_ir(
//                             function,
//                             block,
//                             module,
//                             statement,
//                             next_var,
//                             false,
//                             declarations,
//                             locals,
//                             false,
//                         )? {
//                             return Ok(true);
//                         }
//                     }
//                 } else {
//                     if to_ir(
//                         function,
//                         block,
//                         module,
//                         statement,
//                         next_var,
//                         false,
//                         declarations,
//                         locals,
//                         false,
//                     )? {
//                         return Ok(true);
//                     }
//                 }
//                 i += 1;
//             }
//             Ok(false)
//         }
//         Expr::Index(left, right) => todo!(),
//         Expr::Var(name) => match locals.get(&name).or_else(|| declarations.get(&name)) {
//             None => {
//                 return Err(IRErrorInfo {
//                     idx: expr.idx,
//                     error: IRError::SymbolUndefined(name),
//                 });
//             }
//             Some(Declaration::Variable(number)) => {
//                 if store {
//                     let typ = function.variable_types[*number].clone();
//                     function.variable_types.push(typ);
//                     function.ir[*block].statements.push(Statement::Operation(
//                         Operation::LoadLocal { src: *number },
//                         Some(*next_var),
//                     ));
//                 }
//                 Ok(false)
//             }
//             a => todo!("{a:?}"),
//         },
//         Expr::Literal(literal) => match literal {
//             Literal::String(str) => {
//                 module
//                     .constants
//                     .push((Type::Array(Box::new(Type::u8), str.len()), str.into_bytes()));
//                 if store {
//                     function.ir[*block].statements.push(Statement::Operation(
//                         Operation::LoadGlobal {
//                             src: module.constants.len() - 1,
//                         },
//                         Some(*next_var),
//                     ));
//                     function
//                         .variable_types
//                         .push(Type::Slice(Box::new(Type::u8)));
//                 } else {
//                     function.ir[*block].statements.push(Statement::Operation(
//                         Operation::LoadGlobal {
//                             src: module.constants.len() - 1,
//                         },
//                         None,
//                     ));
//                 }
//                 Ok(false)
//             }
//             Literal::Number(num) => {
//                 module.constants.push((Type::u8, vec![num]));
//                 if store {
//                     function.ir[*block].statements.push(Statement::Operation(
//                         Operation::LoadGlobal {
//                             src: module.constants.len() - 1,
//                         },
//                         Some(*next_var),
//                     ));
//                     function.variable_types.push(Type::u8);
//                 } else {
//                     function.ir[*block].statements.push(Statement::Operation(
//                         Operation::LoadGlobal {
//                             src: module.constants.len() - 1,
//                         },
//                         None,
//                     ));
//                 }
//                 Ok(false)
//             }
//         },
//         Expr::Call(callee, args) => {
//             // TODO: check functions exist and have valid arguments
//             let mut name = Vec::new();

//             let callee = *callee;

//             let mut arg_indexes = Vec::new();
//             for arg in args {
//                 to_ir(
//                     function,
//                     block,
//                     module,
//                     arg,
//                     next_var,
//                     true,
//                     declarations,
//                     locals,
//                     false,
//                 )?;
//                 arg_indexes.push(*next_var);
//                 *next_var += 1;
//             }

//             if mangle_name(&mut name, &callee) {
//                 let sig = match get_declaration(&name, &declarations, callee.idx)? {
//                     Declaration::Function(sig) => sig,
//                     _ => {
//                         return Err(IRErrorInfo {
//                             idx: callee.idx,
//                             error: IRError::SymbolNotCallable(name.last().unwrap().to_string()),
//                         });
//                     }
//                 };
//                 for (arg_index, type_index) in arg_indexes.iter().enumerate() {
//                     if let Some(typ) = sig.args.get(arg_index) {
//                         if function.variable_types[*type_index] != *typ {
//                             return Err(IRErrorInfo {
//                                 idx: callee.idx,
//                                 error: IRError::TypeMismatch {
//                                     got: function.variable_types[*type_index].clone(),
//                                     expected: sig.args[arg_index].clone(),
//                                 },
//                             });
//                         }
//                     } else {
//                         return Err(IRErrorInfo {
//                             idx: expr.idx,
//                             error: IRError::ExtraArgument(),
//                         });
//                     }
//                 }
//                 // TODO: use actual variable list to figure out function types
//                 if store {
//                     function.variable_types.push(sig.returns.clone());
//                     function.ir[*block].statements.push(Statement::Operation(
//                         Operation::Call {
//                             function: name,
//                             args: arg_indexes,
//                         },
//                         Some(*next_var),
//                     ));
//                 } else {
//                     function.ir[*block].statements.push(Statement::Operation(
//                         Operation::Call {
//                             function: name,
//                             args: arg_indexes,
//                         },
//                         None,
//                     ));
//                 }
//             } else {
//                 let function_idx = callee.idx;
//                 let function_var = *next_var;
//                 if to_ir(
//                     function,
//                     block,
//                     module,
//                     callee,
//                     next_var,
//                     true,
//                     declarations,
//                     locals,
//                     false,
//                 )? {
//                     return Ok(true);
//                 }
//                 *next_var += 1;
//                 let sig = match &function.variable_types[function_var] {
//                     Type::Pointer(Pointer::Function(sig)) => sig,
//                     ty => {
//                         return Err(IRErrorInfo {
//                             idx: function_idx,
//                             error: IRError::ExpressionNotCallable(ty.clone()),
//                         });
//                     }
//                 };
//                 for (arg_index, type_index) in arg_indexes.iter().enumerate() {
//                     if function.variable_types[*type_index] != sig.args[arg_index] {
//                         return Err(IRErrorInfo {
//                             idx: function_idx,
//                             error: IRError::TypeMismatch {
//                                 got: function.variable_types[*type_index].clone(),
//                                 expected: sig.args[arg_index].clone(),
//                             },
//                         });
//                     }
//                 }

//                 function.variable_types[*next_var] = sig.returns.clone();
//                 if store {
//                     function.ir[*block].statements.push(Statement::Operation(
//                         Operation::CallPointer {
//                             pointer: function_var,
//                             args: arg_indexes,
//                         },
//                         Some(*next_var),
//                     ));
//                 } else {
//                     function.ir[*block].statements.push(Statement::Operation(
//                         Operation::CallPointer {
//                             pointer: function_var,
//                             args: arg_indexes,
//                         },
//                         None,
//                     ));
//                 }
//             }
//             Ok(false)
//         }
//     };
//     if evaluate {
//         function.ir[*block].terminal =
//             Terminal::Evaluate(if store { Some(*next_var) } else { None });
//     }
//     a
// }

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

pub fn to_string(
    module: &Module,
    blocks: &Vec<Block>,
    vars: HashMap<usize, Option<Vec<u8>>>,
    indentation: usize,
) -> String {
    let mut out = String::new();
    for (idx, block) in blocks.iter().enumerate() {
        for _ in 0..indentation {
            out.push_str("\t");
        }
        out.push_str(&format!("{idx}:\n"));
        for stmt in &block.statements {
            for _ in 0..indentation + 1 {
                out.push_str("\t");
            }
            match stmt {
                Statement::Operation(op, store) => {
                    if let Some(store) = store {
                        out.push_str("$");
                        out.push_str(&store.to_string());
                        out.push_str(" = ");
                    }
                    match op {
                        Operation::Call { function, args } => {
                            let function = function.join(".");
                            out.push_str(&format!("call {function:?}{args:?}"));
                        }
                        Operation::CallPointer { pointer, args } => todo!(),
                        Operation::LoadGlobal { src } => {
                            out.push_str("global \"");
                            out.push_str(
                                &String::from_utf8(module.constants[*src].clone()).unwrap(),
                            );
                            out.push('\"');
                        }
                        Operation::LoadLocal { src } => {
                            out.push('$');
                            out.push_str(&src.to_string());
                        }
                        Operation::Phi { block_to_var } => {
                            out.push_str(&format!("phi {block_to_var:?}"));
                        }
                    }
                }
            }
            out.push('\n');
        }
        for _ in 0..indentation + 1 {
            out.push_str("\t");
        }
        match block.terminal {
            Terminal::CondJump { cond, then, els } => {
                out.push_str(&format!("if ${} then {} else {}", cond, then, els));
            }
            Terminal::Jump(target) => {
                out.push_str(&format!("jump {}", target));
            }
            Terminal::Return(ret) => {
                out.push_str(&format!(
                    "return {}",
                    match ret {
                        Some(var) => format!("${}", var),
                        None => "void".to_string(),
                    }
                ));
            }
            Terminal::Evaluate(ret) => {
                out.push_str(&format!(
                    "evaluate {}",
                    match ret {
                        Some(var) => format!("${}", var),
                        None => "void".to_string(),
                    }
                ));
            }
            Terminal::ThenElseJump(tej) => {
                out.push_str(&format!("then else jump {tej:?}"));
            }
        }
        out.push('\n');
    }
    out
}
