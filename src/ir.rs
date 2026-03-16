use std::{collections::HashMap, fmt::Debug};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{
    expression_parser::{Expr, InfoExpr},
    typ::{Signature, Type},
    value::{EmptyTuple, PrevalValue, Value},
    vm::{RunResult, evaluate},
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
    pub objects: HashMap<String, Value>,
    pub structs: HashMap<String, StructDescriptor>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct StructDescriptor {
    pub fields: IndexMap<String, Type>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct Function {
    pub ir: Vec<Block>,
    pub exported: bool,
    pub signature: Signature,
}

impl PrevalValue for Function {
    fn get_type(&self) -> Type {
        Type::Function(Box::new(self.signature.clone()))
    }

    fn vcall(&self, module: &Module, args: Vec<&Option<Value>>) -> RunResult {
        let mut args_map = HashMap::new();
        for (i, arg) in args.iter().enumerate() {
            args_map.insert(i, arg.clone().clone());
        }
        evaluate(module, self.ir.clone(), &mut args_map, 0)
    }
}

#[derive(PartialEq, Clone, Serialize, Deserialize)]
pub struct Partial {
    pub blocks: Vec<Block>,
}
impl PrevalValue for Partial {
    fn get_type(&self) -> Type {
        Type::Partial
    }

    fn vcall(&self, module: &Module, args: Vec<&Option<Value>>) -> RunResult {
        let mut args_map = HashMap::new();
        for (i, arg) in args.iter().enumerate() {
            args_map.insert(i, arg.clone().clone());
        }
        evaluate(module, self.blocks.clone(), &mut args_map, 0)
    }
}
impl Debug for Partial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&to_string(&self.blocks, 1))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Statement {
    Operation(Operation, Option<usize>),
    Delete(usize),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Operation {
    Call {
        function: Callable,
        args: Vec<usize>,
    },
    LoadLiteral(Value),
    LoadLocal {
        src: usize,
    },
    Phi {
        block_to_var: HashMap<usize, usize>,
    },
    Index(usize, usize),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Callable {
    Var(usize),
    Partial(Value),
}

#[derive(Debug)]
pub enum Declaration {
    Variable(usize),
    Constant,
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
    next_var: &mut usize,
) -> Result<(), IRErrorInfo> {
    match expr.expr {
        Expr::Literal(lit) => {
            if let Some(store) = store {
                function.ir[*block].statements.push(Statement::Operation(
                    Operation::LoadLiteral(lit),
                    Some(store),
                ));
            }
            Ok(())
        }
        Expr::Let(name, value_expr) => {
            let new_var = {
                *next_var += 1;
                *next_var
            };
            to_ir(
                function,
                block,
                module,
                *value_expr,
                Some(new_var),
                declarations,
                locals,
                next_var,
            )?;
            locals.insert(name, Declaration::Variable(new_var));
            if let Some(store) = store {
                function.ir[*block].statements.push(Statement::Operation(
                    Operation::LoadLocal { src: new_var },
                    Some(store),
                ));
            }
            Ok(())
        }
        Expr::Block(statements, returns) => {
            let delete_group_start = *next_var + 1;
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
                        next_var,
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
                        next_var,
                    )?;
                }
                i += 1;
            }

            let delete_group_end = *next_var + 1;

            for i in delete_group_start..delete_group_end {
                function.ir[*block].statements.push(Statement::Delete(i));
            }

            if (len == 0 || !returns) && store.is_some() {
                function.ir[*block].statements.push(Statement::Operation(
                    Operation::LoadLiteral(Value::new(EmptyTuple {})),
                    store,
                ));
            }
            Ok(())
        }
        Expr::Return(value_expr) => {
            function.ir[*block].terminal = Terminal::Return(if let Some(value_expr) = value_expr {
                let return_var = {
                    *next_var += 1;
                    *next_var
                };
                to_ir(
                    function,
                    block,
                    module,
                    *value_expr,
                    Some(return_var),
                    declarations,
                    locals,
                    next_var,
                )?;
                Some(return_var)
            } else {
                None
            });
            Ok(())
        }
        Expr::Call(callee, args) => {
            let callee = *callee;

            let mut arg_indexes = Vec::new();
            for arg in args {
                let i = {
                    *next_var += 1;
                    *next_var
                };
                to_ir(
                    function,
                    block,
                    module,
                    arg,
                    Some(i),
                    declarations,
                    locals,
                    next_var,
                )?;
                arg_indexes.push(i);
            }

            let fn_var = {
                *next_var += 1;
                *next_var
            };
            to_ir(
                function,
                block,
                module,
                callee,
                Some(fn_var),
                declarations,
                locals,
                next_var,
            )?;

            function.ir[*block].statements.push(Statement::Operation(
                Operation::Call {
                    function: Callable::Var(fn_var),
                    args: arg_indexes,
                },
                store,
            ));
            Ok(())
        }
        Expr::Var(name) => {
            if let Some(store) = store {
                match locals.get(&name).or(declarations.get(&name)) {
                    None => {
                        return Err(IRErrorInfo {
                            idx: expr.idx,
                            error: IRError::SymbolUndefined(name),
                        });
                    }
                    Some(Declaration::Variable(v)) => {
                        function.ir[*block].statements.push(Statement::Operation(
                            Operation::LoadLocal { src: *v },
                            Some(store),
                        ));
                    }
                    Some(Declaration::Constant) => {
                        function.ir[*block].statements.push(Statement::Operation(
                            Operation::LoadLiteral(
                                module.objects.get(&name).expect("Valid constant").clone(),
                            ),
                            Some(store),
                        ));
                    }
                    _ => {
                        return Err(IRErrorInfo {
                            idx: expr.idx,
                            error: IRError::NotStorable(name),
                        });
                    }
                }
            }
            Ok(())
        }
        Expr::If { cond, then, els } => {
            // println!("IF: {store:?}");

            let cond_var = {
                *next_var += 1;
                *next_var
            };
            to_ir(
                function,
                block,
                module,
                *cond,
                Some(cond_var),
                declarations,
                locals,
                next_var,
            )?;

            let then_block_n = function.ir.len();
            let mut then_block_n_mut = function.ir.len();
            let then_block_var = {
                *next_var += 1;
                *next_var
            };
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
                next_var,
            )?;

            let else_block = if let Some(els) = els {
                let else_block_n = function.ir.len();
                let mut else_block_n_mut = function.ir.len();
                let else_block_var = {
                    *next_var += 1;
                    *next_var
                };
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
                    next_var,
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
                } else {
                    return Err(IRErrorInfo {
                        idx: expr.idx,
                        error: IRError::MissingElseBlock(),
                    });
                }
            }

            Ok(())
        }
        Expr::Index(left, right) => {
            let left_var = {
                *next_var += 1;
                *next_var
            };
            to_ir(
                function,
                block,
                module,
                *left,
                Some(left_var),
                declarations,
                locals,
                next_var,
            )?;
            let right_var = {
                *next_var += 1;
                *next_var
            };
            to_ir(
                function,
                block,
                module,
                *right,
                Some(right_var),
                declarations,
                locals,
                next_var,
            )?;

            function.ir[*block].statements.push(Statement::Operation(
                Operation::Index(left_var, right_var),
                store,
            ));

            Ok(())
        }
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
                        // Declaration::Module(module) => get_declaration(&names[1..], module, idx),
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

pub fn module_to_string(module: &Module) -> String {
    let mut out = String::new();
    for (name, fun) in &module.objects {
        out.push_str(&name);
        out.push('\n');
        out.push_str(&format!("{fun:?}"));
    }
    out
}

impl Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&to_string(&self.ir, 1))
    }
}

pub fn to_string(blocks: &Vec<Block>, indentation: usize) -> String {
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
                            out.push_str(&format!("call {function:?}{args:?}"));
                        }
                        Operation::LoadLiteral(lit) => {
                            out.push_str(&format!("{lit:?}"));
                        }
                        Operation::LoadLocal { src } => {
                            out.push('$');
                            out.push_str(&src.to_string());
                        }
                        Operation::Phi { block_to_var } => {
                            out.push_str(&format!("phi {block_to_var:?}"));
                        }
                        Operation::Index(left, right) => {
                            out.push_str(&format!("${left}[${right}]"));
                        }
                    }
                }
                Statement::Delete(var) => {
                    out.push_str(&format!("delete ${var}"));
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
        }
        out.push('\n');
    }
    out
}
