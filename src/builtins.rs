use std::{collections::HashMap, ffi::OsString, fs, str::FromStr};

use libloading::{Library, Symbol, library_filename};

use crate::{
    ir::Statement,
    typ::{Signature, Type},
};

pub trait Builtin {
    fn get_signature(&self) -> Signature;

    fn call(
        &self,
        vars: &mut HashMap<usize, Option<Vec<u8>>>,
        args: &Vec<usize>,
        store: &Option<usize>,
        out: &mut Vec<Statement>,
        stmt: &Statement,
    ) {
    }
}

pub fn get_builtins() -> HashMap<String, Box<dyn Builtin>> {
    let mut map: HashMap<String, Box<dyn Builtin>> = HashMap::new();
    map.insert(String::from("print"), Box::new(Print {}));
    map.insert(String::from("read_file"), Box::new(ReadFile {}));
    map.insert(String::from("call_native"), Box::new(CallNative {}));

    map
}

pub struct Print {}
impl Builtin for Print {
    fn get_signature(&self) -> Signature {
        Signature {
            args: vec![Type::IO, Type::Slice(Box::new(Type::u8))],
            returns: Type::Tuple(vec![]),
        }
    }

    fn call(
        &self,
        vars: &mut HashMap<usize, Option<Vec<u8>>>,
        args: &Vec<usize>,
        store: &Option<usize>,
        out: &mut Vec<Statement>,
        stmt: &Statement,
    ) {
        if let Some(Some(_)) = vars.get(&args[0]).clone() {
            if let Some(Some(message)) = vars.get(&args[1]).clone() {
                println!("{}", String::from_utf8(message.to_vec()).unwrap())
            } else {
                out.push(stmt.clone());
            }
        } else {
            out.push(stmt.clone());
        }
    }
}
pub struct ReadFile {}
impl Builtin for ReadFile {
    fn get_signature(&self) -> Signature {
        Signature {
            args: vec![Type::IO, Type::Slice(Box::new(Type::u8))],
            returns: Type::Slice(Box::new(Type::u8)),
        }
    }

    fn call(
        &self,
        vars: &mut HashMap<usize, Option<Vec<u8>>>,
        args: &Vec<usize>,
        store: &Option<usize>,
        out: &mut Vec<Statement>,
        stmt: &Statement,
    ) {
        if let Some(Some(_)) = vars.get(&args[0]).clone() {
            if let Some(Some(path)) = vars.get(&args[1]).clone() {
                let contents = fs::read(String::from_utf8(path.clone()).unwrap()).unwrap();
                if let Some(store) = store {
                    vars.insert(*store, Some(contents));
                }
            } else {
                out.push(stmt.clone());
            }
        } else {
            out.push(stmt.clone());
        }
    }
}

struct CallNative {}
impl Builtin for CallNative {
    fn get_signature(&self) -> Signature {
        Signature {
            args: vec![
                Type::Slice(Box::new(Type::u8)),
                Type::Slice(Box::new(Type::u8)),
            ],
            returns: Type::Tuple(Vec::new()),
        }
    }

    fn call(
        &self,
        vars: &mut HashMap<usize, Option<Vec<u8>>>,
        args: &Vec<usize>,
        store: &Option<usize>,
        out: &mut Vec<Statement>,
        stmt: &Statement,
    ) {
        if let Some(Some(libname)) = vars.get(&args[0]) {
            if let Some(Some(function_name)) = vars.get(&args[1]) {
                unsafe {
                    let lib = Library::new(library_filename(
                        OsString::from_str(&String::from_utf8(libname.clone()).unwrap()).unwrap(),
                    ))
                    .unwrap();
                    let fun: Symbol<unsafe extern "C" fn()> = lib.get(&function_name).unwrap();
                    fun();
                }
            } else {
                out.push(stmt.clone());
            }
        } else {
            out.push(stmt.clone());
        }
    }
}
