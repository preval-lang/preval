use std::{collections::HashMap, fs};

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
