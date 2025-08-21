use std::{
    collections::HashMap,
    fmt::Display,
    ops::{Add, Sub},
};

use crate::{
    mathop::MathOp,
    mem::{MemoryAddress, MemoryReference},
};

pub struct Unit {
    pub sections: HashMap<String, HashMap<String, Label>>,
}

pub struct Label {
    pub global: bool,
    pub contents: HashMap<Option<String>, Vec<PsuedoInstruction>>,
}

impl Label {
    pub fn data(global: bool, data: Vec<u8>) -> Label {
        let mut contents = HashMap::new();
        contents.insert(None, vec![PsuedoInstruction::Bytes(data)]);
        Label { global, contents }
    }
}

#[derive(Clone, Copy)]
pub struct IntegerRegister(pub &'static str);

impl Add<isize> for IntegerRegister {
    type Output = MemoryAddress;

    fn add(self, rhs: isize) -> Self::Output {
        MemoryAddress {
            offset: rhs,
            register: self,
            label_offset: None,
        }
    }
}

impl Sub<isize> for IntegerRegister {
    type Output = MemoryAddress;

    fn sub(self, rhs: isize) -> Self::Output {
        MemoryAddress {
            offset: -rhs,
            register: self,
            label_offset: None,
        }
    }
}

impl Add<&str> for IntegerRegister {
    type Output = MemoryAddress;

    fn add(self, rhs: &str) -> Self::Output {
        MemoryAddress {
            offset: 0,
            register: self,
            label_offset: Some(rhs.to_string()),
        }
    }
}

impl Add<String> for IntegerRegister {
    type Output = MemoryAddress;

    fn add(self, rhs: String) -> Self::Output {
        MemoryAddress {
            offset: 0,
            register: self,
            label_offset: Some(rhs),
        }
    }
}

pub enum PsuedoInstruction {
    Bytes(Vec<u8>),

    ToRegisterMoveMemory(IntegerRegister, MemoryReference),
    ToMemoryMoveRegister(MemoryReference, IntegerRegister),
    ToRegisterMoveRegister(IntegerRegister, IntegerRegister),
    ToRegisterMoveImmediate(IntegerRegister, isize),

    Lea(IntegerRegister, MemoryAddress),

    Add(MathOp),
    Sub(MathOp),
    Mul(MathOp),
    Div(MathOp),

    Call(String),
    Jmp(String),

    Ret,
    Leave,

    PushIntegerRegister(IntegerRegister),
    PushMemory(MemoryReference),
    PushImmediate(isize),
}

pub struct PsuedoInstructionBuilder {
    pub contents: Vec<PsuedoInstruction>,
}

impl From<[IntegerRegister; 1]> for MemoryAddress {
    fn from(value: [IntegerRegister; 1]) -> Self {
        MemoryAddress {
            register: value[0],
            offset: 0,
            label_offset: None,
        }
    }
}

impl From<[MemoryAddress; 1]> for MemoryAddress {
    fn from(value: [MemoryAddress; 1]) -> Self {
        value[0].clone()
    }
}

impl PsuedoInstructionBuilder {
    pub fn lea<'a>(&'a mut self, dest: IntegerRegister, src: impl Into<MemoryAddress>) {
        self.contents.push(PsuedoInstruction::Lea(dest, src.into()));
    }

    pub fn ret<'a>(&'a mut self) {
        self.contents.push(PsuedoInstruction::Ret);
    }

    pub fn leave<'a>(&'a mut self) {
        self.contents.push(PsuedoInstruction::Leave);
    }

    pub fn call<'a>(&'a mut self, name: String) {
        self.contents.push(PsuedoInstruction::Call(name));
    }

    pub fn jmp<'a>(&'a mut self, name: String) {
        self.contents.push(PsuedoInstruction::Jmp(name));
    }
}

impl Unit {
    pub fn to_gas_string(&self) -> String {
        let mut out = String::new();

        out.push_str(".intel_syntax noprefix\n");

        for (name, section) in &self.sections {
            out.push_str(&format!(".section {name}\n"));
            for (name, label) in section {
                if label.global {
                    out.push_str(".global ");
                    out.push_str("\"");
                    out.push_str(name);
                    out.push_str("\"\n");
                }

                out.push_str("\"");
                out.push_str(name);
                out.push_str("\":\n");

                if let Some(pis) = label.contents.get(&None) {
                    for pi in pis {
                        let mut ins = pi.to_gas_string();
                        ins.push('\n');
                        out.push_str(&ins);
                    }
                }

                for (local_name, pis) in &label.contents {
                    if let Some(local_name) = local_name {
                        out.push_str("\"");
                        out.push_str(name);
                        out.push_str("/");
                        out.push_str(local_name);
                        out.push_str("\":\n");
                        for pi in pis {
                            let mut ins = pi.to_gas_string();
                            ins.push('\n');
                            out.push_str(&ins);
                        }
                    }
                }
            }
        }

        out
    }

    pub fn new() -> Unit {
        Unit {
            sections: HashMap::new(),
        }
    }
}
