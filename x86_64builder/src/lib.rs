pub mod mathop;
pub mod mem;
pub mod mov;
pub mod push;
pub mod to_string;
pub mod unit;

pub mod prelude {
    use crate::{
        mem::{MemoryAddress, MemoryReference, WordSize},
        unit::IntegerRegister,
    };

    pub fn qword(addr: impl Into<MemoryAddress>) -> MemoryReference {
        MemoryReference {
            address: addr.into(),
            size: WordSize::qword,
        }
    }

    pub const rax: IntegerRegister = IntegerRegister("rax");
    pub const rdi: IntegerRegister = IntegerRegister("rdi");
    pub const rsi: IntegerRegister = IntegerRegister("rsi");
    pub const rdx: IntegerRegister = IntegerRegister("rdx");
    pub const rcx: IntegerRegister = IntegerRegister("rcx");
    pub const r8: IntegerRegister = IntegerRegister("r8");
    pub const r9: IntegerRegister = IntegerRegister("r9");
    pub const r10: IntegerRegister = IntegerRegister("r10");
    pub const rbp: IntegerRegister = IntegerRegister("rbp");
    pub const rsp: IntegerRegister = IntegerRegister("rsp");
    pub const rip: IntegerRegister = IntegerRegister("rip");
}
