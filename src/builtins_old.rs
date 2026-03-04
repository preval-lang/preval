use std::{
    alloc::{Layout, alloc},
    cell::RefCell,
    collections::HashMap,
    ffi::{CStr, CString, OsString, c_void},
    fs,
    io::Write,
    mem::transmute,
    str::FromStr,
};

use libffi::{
    high::{self, CType, arg},
    middle::{self, Arg, Cif},
};
use libloading::{Library, Symbol, library_filename};

use crate::{
    ir_old::{Statement, Value},
    typ::{Signature, Type},
};

pub trait Builtin {
    fn get_signature(&self) -> Signature;

    fn call(
        &self,
        vars: &mut HashMap<usize, Option<Value>>,
        generics: &Vec<Type>,
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
            args: vec![Type::IO, Type::string],
            returns: Type::Tuple(vec![]),
            generics: Vec::new(),
        }
    }

    fn call(
        &self,
        vars: &mut HashMap<usize, Option<Value>>,
        generics: &Vec<Type>,
        args: &Vec<usize>,
        store: &Option<usize>,
        out: &mut Vec<Statement>,
        stmt: &Statement,
    ) {
        if let Some(Some(_)) = vars.get(&args[0]).clone() {
            if let Some(Some(Value::string(message))) = vars.get(&args[1]).clone() {
                println!("{}", message);
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
            args: vec![Type::IO, Type::string],
            returns: Type::string,
            generics: Vec::new(),
        }
    }

    fn call(
        &self,
        vars: &mut HashMap<usize, Option<Value>>,
        generics: &Vec<Type>,
        args: &Vec<usize>,
        store: &Option<usize>,
        out: &mut Vec<Statement>,
        stmt: &Statement,
    ) {
        if let Some(Some(_)) = vars.get(&args[0]).clone() {
            if let Some(Some(Value::string(path))) = vars.get(&args[1]).clone() {
                let contents = fs::read(path).unwrap();
                if let Some(store) = store {
                    vars.insert(
                        *store,
                        Some(Value::string(String::from_utf8(contents).unwrap())),
                    );
                }
            } else {
                out.push(stmt.clone());
            }
        } else {
            out.push(stmt.clone());
        }
    }
}

#[repr(C)]
struct NativeResult {
    data: *mut u8,
    len: usize,
}

struct CallNative {}
impl Builtin for CallNative {
    fn get_signature(&self) -> Signature {
        Signature {
            args: vec![Type::string, Type::string, Type::Generic(1)],
            returns: Type::Generic(0),
            generics: vec![(), ()],
        }
    }

    fn call(
        &self,
        vars: &mut HashMap<usize, Option<Value>>,
        generics: &Vec<Type>,
        args: &Vec<usize>,
        store: &Option<usize>,
        out: &mut Vec<Statement>,
        stmt: &Statement,
    ) {
        if let Some(Some(Value::string(libname))) = vars.get(&args[0]) {
            if let Some(Some(Value::string(function_name))) = vars.get(&args[1]) {
                if let Some(Some(args)) = vars.get(&args[2]) {
                    unsafe {
                        let lib =
                            Library::new(library_filename(OsString::from_str(libname).unwrap()))
                                .unwrap();
                        let fun: Symbol<unsafe extern "C" fn()> =
                            lib.get(&function_name.as_bytes()).unwrap();

                        let return_type = type_to_ctype(generics[0].clone());

                        let (c_args_types, c_args) = match args {
                            Value::Tuple(contents) => {
                                let cloned = contents
                                    .iter()
                                    .map(Clone::clone)
                                    .map(c_ify)
                                    .filter_map(|f| f);
                                (
                                    cloned
                                        .clone()
                                        .map(|f| f.as_arg_types())
                                        .collect::<Vec<_>>()
                                        .concat(),
                                    cloned.collect::<Vec<_>>(),
                                )
                            }
                            _ => panic!("arg #3 is not a tuple"),
                        };

                        let cif = Cif::new(c_args_types, return_type);

                        let mut args_final = Vec::new();

                        for cffivalue in &c_args {
                            match cffivalue {
                                CFFIValue::u8(u8) => {
                                    args_final.push(Arg::new(u8));
                                }
                                CFFIValue::string(string) => {
                                    args_final.push(Arg::new(string));
                                }
                                CFFIValue::bool(bool) => {
                                    args_final.push(Arg::new(bool));
                                }
                                CFFIValue::Tuple(tuple) => {
                                    todo!()
                                }
                                CFFIValue::Slice(slice) => {
                                    args_final.push(Arg::new(slice));
                                }
                            }
                        }

                        let rv = cif.call::<*const i8>(
                            transmute(fun.try_as_raw_ptr().unwrap()),
                            &args_final,
                        );

                        todo!("Non string return values");

                        if let Some(store) = store {
                            vars.insert(
                                *store,
                                Some(Value::string(
                                    CStr::from_ptr(rv).to_str().unwrap().to_string(),
                                )),
                            );
                        }
                    }
                } else {
                    out.push(stmt.clone());
                }
            } else {
                out.push(stmt.clone());
            }
        } else {
            out.push(stmt.clone());
        }
    }
}

struct CFFIValueWithArg<'a> {
    cffivalue: CFFIValue,
    arg: Arg<'a>,
}

#[repr(C)]
struct Slice {
    len: usize,
    data: *mut c_void,
}

enum CFFIValue {
    Slice(Slice),
    bool(bool),
    string(CString),
    u8(u8),
    Tuple(Vec<CFFIValue>),
}

impl CFFIValue {
    fn write(&self, buf: &mut Vec<u8>) {
        match self {
            CFFIValue::Slice(slice) => buf.extend(unsafe { any_as_u8_slice(&slice) }),
            CFFIValue::Tuple(tuple) => {
                for item in tuple {
                    item.write(buf);
                }
            }
            CFFIValue::bool(bool) => {
                buf.push(if *bool { 1 } else { 0 });
            }
            CFFIValue::u8(u8) => {
                buf.push(*u8);
            }
            CFFIValue::string(str) => {
                buf.write(str.as_bytes_with_nul());
            }
        }
    }

    fn as_arg_types<'a>(&self) -> Vec<middle::Type> {
        match self {
            CFFIValue::Slice(slice) => {
                vec![middle::Type::structure(vec![
                    middle::Type::usize(),
                    middle::Type::pointer(),
                ])]
            }
            CFFIValue::Tuple(values) => {
                let mut out = Vec::new();
                for value in values {
                    out.extend(value.as_arg_types());
                }
                out
            }
            CFFIValue::bool(u8) => vec![middle::Type::u8()],
            CFFIValue::string(cstr) => vec![middle::Type::pointer()],
            CFFIValue::u8(u8) => vec![middle::Type::u8()],
        }
    }
}

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    unsafe {
        ::core::slice::from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>())
    }
}

fn c_ify<'a>(value: Value) -> Option<CFFIValue> {
    match value {
        Value::IO => None,
        Value::Slice(contents) => {
            let mut buf = Vec::new();
            for item in contents.iter().map(Clone::clone).map(c_ify) {
                if let Some(item) = item {
                    item.write(&mut buf);
                }
            }
            Some(CFFIValue::Slice(Slice {
                len: contents.len(),
                data: buf.as_mut_ptr() as *mut c_void,
            }))
        }
        Value::bool(bool) => Some(CFFIValue::bool(bool)),
        Value::string(str) => Some(CFFIValue::string(CString::from_str(&str).unwrap())),
        Value::u8(u8) => Some(CFFIValue::u8(u8)),
        Value::Tuple(tuple) => Some(CFFIValue::Tuple(
            tuple
                .iter()
                .map(Clone::clone)
                .map(c_ify)
                .filter_map(|f| f)
                .collect(),
        )),
    }
}

fn type_to_ctype(typ: Type) -> middle::Type {
    match typ {
        Type::string => middle::Type::pointer(),
        _ => todo!("Type to ctype"),
    }
}
