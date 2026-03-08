use std::{
    any::{Any, type_name},
    collections::HashMap,
    fmt::{Debug, Display},
    ops::Index,
    process::Output,
    sync::Arc,
};

use serde::{Deserialize, Serialize};

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
    fn pre_serialize<'a>(&'a self) -> Option<&'a dyn erased_serde::Serialize>;
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

trait PrevalValue: PreSerialize {
    fn vindex(&self, value: &dyn Value) -> Box<dyn Value> {
        panic!("Not indexable: {}", type_name::<Self>())
    }

    fn deserializer_id(&self) -> Option<String> {
        None
    }
}

trait PreSerialize {
    fn pre_serialize<'a>(&'a self) -> Option<&'a dyn erased_serde::Serialize>;
}

impl<T: erased_serde::Serialize> PreSerialize for T {
    fn pre_serialize<'a>(&'a self) -> Option<&'a dyn erased_serde::Serialize> {
        Some(self)
    }
}

impl<T: PartialEq + Clone + Debug + PrevalValue + 'static> Value for T {
    fn vclone(&self) -> Box<dyn Value> {
        let c = self.clone();
        Box::new(c)
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

    fn pre_serialize<'a>(&'a self) -> Option<&'a dyn erased_serde::Serialize> {
        PreSerialize::pre_serialize(self)
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

impl PrevalValue for usize {
    fn deserializer_id(&self) -> Option<String> {
        Some("usize".to_string())
    }
}

impl PrevalValue for bool {
    fn deserializer_id(&self) -> Option<String> {
        Some("bool".to_string())
    }
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct IO;
impl PrevalValue for IO {
    fn deserializer_id(&self) -> Option<String> {
        Some("nop".to_string())
    }
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct EmptyTuple;
impl PrevalValue for EmptyTuple {
    fn deserializer_id(&self) -> Option<String> {
        Some("nop".to_string())
    }
}
