use std::collections::HashMap;

use x86_64builder::unit::{
    IntegerRegister, Label, PsuedoInstruction, PsuedoInstructionBuilder, Unit,
};

use x86_64builder::prelude::*;

use crate::ir::{Function, Module, Operation, Statement, Type};

impl Type {
    fn size_x86_64(&self) -> isize {
        match self {
            Type::usize | Type::Pointer(_) => 8,
            Type::Slice(_) => 16, // usize+pointer
            Type::void => 0,
            Type::u8 => 1,
            Type::Array(typ, len) => typ.size_x86_64() * (*len as isize),
        }
    }
}

#[derive(PartialEq)]
enum Classification {
    Integer,
    X87,
    Memory,
    SSE,
}

pub fn compile(module: Module) -> String {
    let mut section_data = HashMap::new();

    let mut i = 0;

    for (_, data) in &module.constants {
        section_data.insert(format!("_c.{i}"), Label::data(false, data.to_vec()));
        i += 1;
    }

    let mut section_text = HashMap::new();

    for (name, function) in &module.functions {
        section_text.insert(
            name.to_string(),
            Label {
                global: function.exported,
                contents: {
                    let mut out = HashMap::new();

                    let (stack_size, offsets, prologue_pis) = prologue(&function, name);

                    out.insert(None, prologue_pis);

                    for (i, block) in function.ir.iter().enumerate() {
                        out.insert(
                            Some(i.to_string() + "block"),
                            compile_block(block, &function, &offsets, &module, name),
                        );
                    }

                    out.insert(Some("epilogue".to_string()), epilogue(stack_size));

                    out
                },
            },
        );
    }

    let mut unit = Unit::new();
    unit.sections.insert(".data".to_string(), section_data);
    unit.sections.insert(".text".to_string(), section_text);

    unit.to_gas_string()
}

static INT_ARG_REGISTERS: [IntegerRegister; 6] = [rdi, rsi, rdx, rcx, r8, r9];

fn load_args(
    asm: &mut PsuedoInstructionBuilder,
    offsets: &Vec<isize>,
    args_vars: &Vec<usize>,
    types: &Vec<Type>,
) {
    let mut int_reg_arg = 0;

    let mut args = Vec::new();

    for arg_var in args_vars {
        args.push((offsets[*arg_var], types[*arg_var].clone()));
    }

    let mut stack_passed = Vec::new();

    for (offset, typ) in args {
        match typ {
            Type::Array(a, _) => todo!("Should arrays decay like in C? Probably not"),
            Type::Pointer(_) | Type::u8 | Type::usize => {
                if int_reg_arg < 6 {
                    asm.mov(INT_ARG_REGISTERS[int_reg_arg], qword([rbp - (offset + 8)]));
                    int_reg_arg += 1;
                } else {
                    stack_passed.push(qword([rbp - (offset + 8)]));
                }
            }
            Type::Slice(_) => {
                if int_reg_arg < 5 {
                    asm.mov(INT_ARG_REGISTERS[int_reg_arg], qword([rbp - (offset + 8)]));
                    int_reg_arg += 1;
                    asm.mov(INT_ARG_REGISTERS[int_reg_arg], qword([rbp - (offset + 16)]));
                    int_reg_arg += 1;
                } else {
                    stack_passed.push(qword([rbp - (offset + 8)]));
                    stack_passed.push(qword([rbp - (offset + 16)]));
                }
            }
            Type::void => panic!("VOID!"),
        }
    }
}

fn prologue(function: &Function, name: &String) -> (isize, Vec<isize>, Vec<PsuedoInstruction>) {
    let mut asm = PsuedoInstructionBuilder {
        contents: Vec::new(),
    };

    let mut stack_size: isize = 0;
    let mut offsets = Vec::new();
    for typ in &function.variable_types {
        offsets.push(stack_size);
        stack_size += typ.size_x86_64();
        if stack_size % 8 != 0 {
            stack_size = (stack_size + 7) & !7
        }
    }
    if stack_size % 16 != 0 {
        stack_size = (stack_size + 15) & !15
    }

    asm.push(rbp);
    asm.mov(rbp, rsp);

    asm.sub(rsp, stack_size);

    asm.jmp(name.to_string() + "/0block");

    (stack_size, offsets, asm.contents)
}

fn epilogue(stack_size: isize) -> Vec<PsuedoInstruction> {
    let mut asm = PsuedoInstructionBuilder {
        contents: Vec::new(),
    };

    asm.add(rsp, stack_size);
    asm.leave();
    asm.ret();

    asm.contents
}

fn compile_block(
    block: &Vec<Statement>,
    function: &Function,
    offsets: &Vec<isize>,
    module: &Module,
    name: &String,
) -> Vec<PsuedoInstruction> {
    let mut asm = PsuedoInstructionBuilder {
        contents: Vec::new(),
    };

    for ir in block {
        let (dest, op) = match ir {
            Statement::Stored(op, dest) => (Some(dest), op),
            Statement::Void(op) => (None, op),
        };

        match op {
            Operation::Call {
                function: callee,
                args,
            } => {
                load_args(&mut asm, &offsets, &args, &function.variable_types);
                asm.call(callee.join("."));
                if let Some(dest) = dest {
                    let typ = &function.variable_types[*dest];
                    match typ {
                        Type::Slice(_) => {
                            asm.mov(qword([rbp - (offsets[*dest] + 8)]), rax);
                            asm.mov(qword([rbp - (offsets[*dest] + 16)]), rdx);
                        }
                        _ => todo!("Handle non-slice returns"),
                    }
                }
            }
            Operation::Return(var) => {
                if let Some(var) = var {
                    let typ = &function.variable_types[*var];
                    let offset = offsets[*var];
                    match typ {
                        Type::Slice(_) => {
                            asm.mov(rax, qword([rbp - (offset + 8)]));
                            asm.mov(rdx, qword([rbp - (offset + 16)]));
                        }
                        Type::Array(_, _) => todo!("RETURN ARRAYS"),
                        Type::u8 | Type::Pointer(_) | Type::usize => {
                            asm.mov(rax, qword([rbp - (offset + 8)]));
                        }
                        Type::void => panic!("RETURN VOID"),
                    }
                }
                asm.jmp(name.to_string() + "/epilogue");
            }
            Operation::CallPointer { pointer, args } => todo!(),
            Operation::LoadGlobalSlice { src } => {
                if let Some(dest) = dest {
                    println!("{:?}", module.constants[*src]);
                    asm.lea(rax, [rip + format!("_c.{src}")]);
                    asm.mov(qword([rbp - (offsets[*dest] + 16)]), rax);
                    asm.mov(rax, module.constants[*src].0.size_x86_64());
                    asm.mov(qword([rbp - (offsets[*dest] + 8)]), rax);
                }
            }
        }
    }

    asm.contents
}
