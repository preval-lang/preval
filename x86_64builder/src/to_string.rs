use crate::{mem::MemoryAddress, unit::PsuedoInstruction};

impl PsuedoInstruction {
    pub fn to_gas_string(&self) -> String {
        match self {
            PsuedoInstruction::Add(op)
            | PsuedoInstruction::Div(op)
            | PsuedoInstruction::Mul(op)
            | PsuedoInstruction::Sub(op) => {
                format!(
                    "{} {}",
                    match self {
                        PsuedoInstruction::Add(_) => "add",
                        PsuedoInstruction::Div(_) => "div",
                        PsuedoInstruction::Mul(_) => "mul",
                        PsuedoInstruction::Sub(_) => "sub",
                        _ => unreachable!(),
                    },
                    op.to_gas_string()
                )
            }
            PsuedoInstruction::Jmp(name) => {
                format!("jmp \"{name}\"")
            }
            PsuedoInstruction::Bytes(bytes) => {
                format!(
                    ".byte {}",
                    bytes
                        .iter()
                        .map(|a| a.to_string())
                        .collect::<Vec<String>>()
                        .join(",")
                        .as_str()
                )
            }
            PsuedoInstruction::Call(name) => {
                format!("call \"{name}\"")
            }
            PsuedoInstruction::Lea(reg, mem) => {
                format!("lea {}, {}", reg.0, mem.to_gas_string())
            }
            PsuedoInstruction::Leave => "leave".to_string(),
            PsuedoInstruction::Ret => "ret".to_string(),
            PsuedoInstruction::PushImmediate(imm) => {
                format!("push {imm}")
            }
            PsuedoInstruction::PushIntegerRegister(reg) => {
                format!("push {}", reg.0)
            }
            PsuedoInstruction::PushMemory(mem) => {
                format!("push {}", mem.to_gas_string())
            }
            PsuedoInstruction::ToMemoryMoveRegister(mem, reg) => {
                format!("mov {}, {}", mem.to_gas_string(), reg.0)
            }
            PsuedoInstruction::ToRegisterMoveMemory(reg, mem) => {
                format!("mov {}, {}", reg.0, mem.to_gas_string())
            }
            PsuedoInstruction::ToRegisterMoveRegister(left, right) => {
                format!("mov {}, {}", left.0, right.0)
            }
            PsuedoInstruction::ToRegisterMoveImmediate(reg, imm) => {
                format!("mov {}, {}", reg.0, imm)
            }
        }
    }
}
