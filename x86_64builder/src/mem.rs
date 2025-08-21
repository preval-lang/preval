use crate::unit::IntegerRegister;

#[derive(Clone)]
pub enum WordSize {
    qword,
    dword,
    word,
    byte,
}

impl WordSize {
    pub fn name(&self) -> &'static str {
        match self {
            WordSize::qword => "qword",
            WordSize::dword => "dword",
            WordSize::word => "word",
            WordSize::byte => "byte",
        }
    }
}

#[derive(Clone)]
pub struct MemoryAddress {
    pub register: IntegerRegister,
    pub offset: isize,
    pub label_offset: Option<String>,
}

#[derive(Clone)]
pub struct MemoryReference {
    pub address: MemoryAddress,
    pub size: WordSize,
}

impl MemoryReference {
    pub fn to_gas_string(&self) -> String {
        format!("{} {}", self.size.name(), self.address.to_gas_string())
    }
}

impl MemoryAddress {
    pub fn to_gas_string(&self) -> String {
        let mut out = "[".to_string();
        out.push_str(self.register.0);
        if self.offset > 0 {
            out.push('+');
            out.push_str(&self.offset.to_string());
        } else if self.offset < 0 {
            out.push_str(&self.offset.to_string());
        }
        if let Some(label) = &self.label_offset {
            out.push('+');
            out.push_str(label);
        }
        out.push(']');
        out
    }
}
