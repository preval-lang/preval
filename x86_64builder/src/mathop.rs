use std::fmt::Display;

use crate::{
    mem::{MemoryAddress, MemoryReference},
    unit::{IntegerRegister, PsuedoInstruction, PsuedoInstructionBuilder},
};

pub enum MathOp {
    ToRegisterXRegister(IntegerRegister, IntegerRegister),
    ToRegisterXMemory(IntegerRegister, MemoryReference),
    ToMemoryXRegister(MemoryReference, IntegerRegister),
    ToRegisterXImmediate(IntegerRegister, isize),
    ToMemoryXImmediate(MemoryReference, isize),
}

impl MathOp {
    pub fn to_gas_string(&self) -> String {
        match self {
            MathOp::ToMemoryXImmediate(mem, imm) => {
                format!("{}, {imm}", mem.to_gas_string(),)
            }
            MathOp::ToMemoryXRegister(mem, reg) => {
                format!("{}, {}", mem.to_gas_string(), reg.0)
            }
            MathOp::ToRegisterXImmediate(reg, imm) => {
                format!("{}, {imm}", reg.0)
            }
            MathOp::ToRegisterXMemory(reg, mem) => {
                format!("{}, {}", reg.0, mem.to_gas_string())
            }
            MathOp::ToRegisterXRegister(left, right) => {
                format!("{}, {}", left.0, right.0)
            }
        }
    }
}

trait MathOpableBy<T> {
    fn math_op_by(self, t: T) -> MathOp;
}

impl MathOpableBy<isize> for IntegerRegister {
    fn math_op_by(self, t: isize) -> MathOp {
        MathOp::ToRegisterXImmediate(self, t)
    }
}

impl MathOpableBy<IntegerRegister> for IntegerRegister {
    fn math_op_by(self, t: IntegerRegister) -> MathOp {
        MathOp::ToRegisterXRegister(self, t)
    }
}

impl MathOpableBy<MemoryReference> for IntegerRegister {
    fn math_op_by(self, t: MemoryReference) -> MathOp {
        MathOp::ToRegisterXMemory(self, t)
    }
}

impl MathOpableBy<IntegerRegister> for MemoryReference {
    fn math_op_by(self, t: IntegerRegister) -> MathOp {
        MathOp::ToMemoryXRegister(self, t)
    }
}

impl MathOpableBy<isize> for MemoryReference {
    fn math_op_by(self, t: isize) -> MathOp {
        MathOp::ToMemoryXImmediate(self, t)
    }
}

impl PsuedoInstructionBuilder {
    pub fn add<'a, Left, Right>(&'a mut self, left: Left, right: Right)
    where
        Left: MathOpableBy<Right>,
    {
        self.contents
            .push(PsuedoInstruction::Add(left.math_op_by(right)));
    }

    pub fn sub<'a, Left, Right>(&'a mut self, left: Left, right: Right)
    where
        Left: MathOpableBy<Right>,
    {
        self.contents
            .push(PsuedoInstruction::Sub(left.math_op_by(right)));
    }
}
