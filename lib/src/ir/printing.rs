use crate::ir::{Operation, Partial, Terminal};
use std::fmt::Debug;

use crate::ir::Statement;
use crate::ir::{Block, Function, Module};

pub fn module_to_string(module: &Module) -> String {
    let mut out = String::new();
    for (name, fun) in &module.objects {
        out.push_str(&name);
        out.push('\n');
        out.push_str(&format!("{fun:?}"));
    }
    out
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
                        Operation::InitializeStruct(name, fields) => {
                            out.push_str(&format!("struct {name:?}({fields:?})"));
                        }
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
                        Operation::Access(left, right) => {
                            out.push_str(&format!("${left}.${right}"));
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
        match block.terminal.clone() {
            Terminal::CondJump { cond, then, els } => {
                out.push_str(&format!("if ${} then {} else {}", cond, then, els));
            }
            Terminal::Jump(target) => {
                out.push_str(&format!("jump {}", target));
            }
            Terminal::Return(ret) => {
                out.push_str(&format!("return {}", format!("${}", ret),));
            }
            Terminal::Branch { cond, then, els } => {
                out.push_str(&format!("branch ${} then {:?} else {:?}", cond, then, els));
            }
        }
        out.push('\n');
    }
    out
}

impl Debug for Partial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&to_string(&self.blocks, 1))
    }
}
