use crate::{
    mem::MemoryReference,
    unit::{IntegerRegister, PsuedoInstruction, PsuedoInstructionBuilder},
};

pub trait MovableInto<T> {
    fn mov(self, t: T) -> PsuedoInstruction;
}

impl MovableInto<MemoryReference> for IntegerRegister {
    fn mov(self, t: MemoryReference) -> PsuedoInstruction {
        PsuedoInstruction::ToMemoryMoveRegister(t, self)
    }
}

impl MovableInto<IntegerRegister> for MemoryReference {
    fn mov(self, t: IntegerRegister) -> PsuedoInstruction {
        PsuedoInstruction::ToRegisterMoveMemory(t, self)
    }
}

impl MovableInto<IntegerRegister> for IntegerRegister {
    fn mov(self, t: IntegerRegister) -> PsuedoInstruction {
        PsuedoInstruction::ToRegisterMoveRegister(t, self)
    }
}

impl MovableInto<IntegerRegister> for isize {
    fn mov(self, t: IntegerRegister) -> PsuedoInstruction {
        PsuedoInstruction::ToRegisterMoveImmediate(t, self)
    }
}

impl PsuedoInstructionBuilder {
    pub fn mov<'a, Dest, Src>(&'a mut self, dest: Dest, src: Src)
    where
        Src: MovableInto<Dest>,
    {
        self.contents.push(src.mov(dest));
    }
}
