use std::{
    any::{Any, type_name},
    collections::HashMap,
    fmt::{Debug, Display},
    ops::Index,
    sync::Arc,
};

// #[derive(Debug, Clone, PartialEq)]
// pub enum Value {
//     String(String),
//     Usize(usize),
//     Bool(bool),
//     Char(char),
//     EmptyTuple,
//     IO,
//     Struct(HashMap<String, Value>, Arc<StructDescriptor>),
// }

pub trait Value: Debug {
    fn vclone(&self) -> Box<dyn Value>;
    fn index(&self, value: &dyn Value) -> Box<dyn Value> {
        panic!("Type is not indexable")
    }
    fn vto_string(&self) -> String;
    fn veq(&self, other: &Box<dyn Value>) -> bool;
    fn as_any(&self) -> &dyn Any;
}

impl PartialEq for Box<dyn Value> {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref().veq(other)
    }
}

impl Clone for Box<dyn Value> {
    fn clone(&self) -> Self {
        self.as_ref().vclone()
    }
}

// impl Display for Value {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Value::String(s) => write!(f, "\"{}\"", s),
//             Value::Usize(n) => write!(f, "{}", n),
//             Value::Bool(b) => write!(f, "{}", b),
//             Value::EmptyTuple => write!(f, "()"),
//             Value::IO => write!(f, "IO"),
//             Value::Char(c) => write!(f, "'{c}'"),
//         }
//     }
// }

trait PrevalValue {
    fn vindex(&self, value: &dyn Value) -> Box<dyn Value> {
        panic!("Not indexable: {}", type_name::<Self>())
    }
}

impl<T: PartialEq + Clone + Debug + PrevalValue + 'static> Value for T {
    fn vclone(&self) -> Box<dyn Value> {
        Box::new(self.clone())
    }

    fn veq(&self, other: &Box<dyn Value>) -> bool {
        match other.as_any().downcast_ref::<T>() {
            Some(other) => self == other,
            None => panic!("Can't compare string to non-string"),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn index(&self, value: &dyn Value) -> Box<dyn Value> {
        self.vindex(value)
    }

    fn vto_string(&self) -> String {
        format!("{self:?}")
    }
}

impl PrevalValue for String {
    fn vindex(&self, value: &dyn Value) -> Box<dyn Value> {
        match value.as_any().downcast_ref::<usize>() {
            Some(other) => Box::new(self.chars().nth(*other).unwrap().to_string()),
            None => panic!("Index string with non-usize"),
        }
    }
}

impl PrevalValue for usize {}
impl PrevalValue for bool {}

#[derive(PartialEq, Debug, Clone)]
pub struct IO {}
impl PrevalValue for IO {}

#[derive(PartialEq, Debug, Clone)]
pub struct EmptyTuple {}
impl PrevalValue for EmptyTuple {}
