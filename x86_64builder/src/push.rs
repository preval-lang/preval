use crate::{
    mem::MemoryReference,
    unit::{IntegerRegister, PsuedoInstruction, PsuedoInstructionBuilder},
};

trait Pushable {
    fn push(self) -> PsuedoInstruction;
}

impl Pushable for isize {
    fn push(self) -> PsuedoInstruction {
        PsuedoInstruction::PushImmediate(self)
    }
}

impl Pushable for MemoryReference {
    fn push(self) -> PsuedoInstruction {
        PsuedoInstruction::PushMemory(self)
    }
}

impl Pushable for IntegerRegister {
    fn push(self) -> PsuedoInstruction {
        PsuedoInstruction::PushIntegerRegister(self)
    }
}

impl PsuedoInstructionBuilder {
    pub fn push<'a, Src>(&'a mut self, src: Src)
    where
        Src: Pushable,
    {
        self.contents.push(src.push());
    }
}
