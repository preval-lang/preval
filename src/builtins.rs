use std::{
    collections::{HashMap, HashSet},
    ffi::OsString,
    fs,
    str::FromStr,
};

use libloading::{Library, Symbol, library_filename};

use crate::{
    ir::Statement,
    typ::{Signature, Type},
};

use crate::value::Value;

pub trait Builtin {
    fn get_signature(&self) -> Signature;

    fn call(
        &self,
        vars: &mut HashMap<usize, Option<Value>>,
        args: &Vec<usize>,
        store: &Option<usize>,
        out: &mut Vec<Statement>,
        stmt: &Statement,
        no_delete: &mut HashSet<usize>,
    ) {
    }
}

pub fn get_builtins() -> HashMap<String, Box<dyn Builtin>> {
    let mut map: HashMap<String, Box<dyn Builtin>> = HashMap::new();
    map.insert(String::from("print"), Box::new(Print {}));
    map.insert(String::from("read_file"), Box::new(ReadFile {}));
    // map.insert(String::from("call_native"), Box::new(CallNative {}));

    map
}

pub struct Print {}
impl Builtin for Print {
    fn get_signature(&self) -> Signature {
        Signature {
            args: vec![Type::IO, Type::String],
            returns: Type::Tuple(vec![]),
        }
    }

    fn call(
        &self,
        vars: &mut HashMap<usize, Option<Value>>,
        args: &Vec<usize>,
        store: &Option<usize>,
        out: &mut Vec<Statement>,
        stmt: &Statement,
        no_delete: &mut HashSet<usize>,
    ) {
        match vars.get(&args[0]).clone() {
            Some(Some(_)) => match vars.get(&args[1]).clone() {
                Some(Some(message)) => {
                    if let Some(Some(message)) = vars.get(&args[1]).clone() {
                        println!("{}", message)
                    } else {
                        no_delete.insert(args[0]);
                        no_delete.insert(args[1]);
                        out.push(stmt.clone());
                    }
                }
                Some(None) => {
                    no_delete.insert(args[0]);
                    no_delete.insert(args[1]);
                    out.push(stmt.clone());
                }
                None => {
                    panic!("use of dropped variable");
                }
            },
            Some(None) => {
                no_delete.insert(args[0]);
                no_delete.insert(args[1]);
                out.push(stmt.clone());
            }
            None => {
                panic!("use of dropped variable");
            }
        }
    }
}
pub struct ReadFile {}
impl Builtin for ReadFile {
    fn get_signature(&self) -> Signature {
        Signature {
            args: vec![Type::IO, Type::String],
            returns: Type::String,
        }
    }

    fn call(
        &self,
        vars: &mut HashMap<usize, Option<Value>>,
        args: &Vec<usize>,
        store: &Option<usize>,
        out: &mut Vec<Statement>,
        stmt: &Statement,
        no_delete: &mut HashSet<usize>,
    ) {
        if let Some(Some(_)) = vars.get(&args[0]).clone() {
            if let Some(Some(path)) = vars.get(&args[1]).clone() {
                match path {
                    Value::String(path) => {
                        let contents = fs::read(path).unwrap();
                        if let Some(store) = store {
                            vars.insert(
                                *store,
                                Some(Value::String(String::from_utf8(contents).unwrap())),
                            );
                        }
                    }
                    o => panic!("Incorrect path type"),
                }
            } else {
                no_delete.insert(args[0]);
                no_delete.insert(args[1]);
                out.push(stmt.clone());
            }
        } else {
            no_delete.insert(args[0]);
            no_delete.insert(args[1]);
            out.push(stmt.clone());
        }
    }
}

// struct CallNative {}
// impl Builtin for CallNative {
//     fn get_signature(&self) -> Signature {
//         Signature {
//             args: vec![
//                 Type::Slice(Box::new(Type::u8)),
//                 Type::Slice(Box::new(Type::u8)),
//             ],
//             returns: Type::Tuple(Vec::new()),
//         }
//     }

//     fn call(
//         &self,
//         vars: &mut HashMap<usize, Option<Value>>,
//         args: &Vec<usize>,
//         store: &Option<usize>,
//         out: &mut Vec<Statement>,
//         stmt: &Statement,
//     ) {
//         if let Some(Some(libname)) = vars.get(&args[0]) {
//             if let Some(Some(function_name)) = vars.get(&args[1]) {
//                 unsafe {
//                     let lib = Library::new(library_filename(
//                         OsString::from_str(&String::from_utf8(libname.clone()).unwrap()).unwrap(),
//                     ))
//                     .unwrap();
//                     let fun: Symbol<unsafe extern "C" fn()> = lib.get(&function_name).unwrap();
//                     fun();
//                 }
//             } else {
//                 out.push(stmt.clone());
//             }
//         } else {
//             out.push(stmt.clone());
//         }
//     }
// }
