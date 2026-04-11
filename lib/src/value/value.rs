use std::{
    any::{Any, type_name},
    collections::{HashMap, HashSet},
    fmt::Debug,
};

use serde::{Deserialize, Serialize};

use crate::{
    ir::{Block, Module},
    value::{
        primitive::EmptyTuple,
        typ::{Type, deserialize_type},
    },
    vm::RunResult,
};

#[repr(C)]
#[derive(Clone)]
pub struct Value {
    pub typ: Type,
    pub data: Box<dyn ValueData>,
}

impl Value {
    pub fn new<T: ValueData>(value: T) -> Value {
        Value {
            typ: value.get_type(),
            data: value.vclone(),
        }
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.data, f)
    }
}

pub trait ValueData: Debug {
    fn vclone(&self) -> Box<dyn ValueData>;
    fn index(&mut self, value: &Value) -> Value {
        panic!("Type is not indexable")
    }
    fn call(&mut self, module: &Module, args: Vec<&Option<Value>>) -> RunResult;
    fn vto_string(&self) -> String;
    fn veq(&self, other: &Value) -> bool;
    fn as_any(&self) -> &dyn Any;
    fn pre_serialize<'a>(&'a self) -> Option<&'a dyn erased_serde::Serialize>;
    fn get_type(&self) -> Type;
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        self.data.veq(other)
    }
}

impl Clone for Box<dyn ValueData> {
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

pub trait PrevalValue: PreSerialize {
    fn vindex(&mut self, value: &Value) -> Value {
        panic!("Not indexable: {}", type_name::<Self>())
    }

    fn vcall(&mut self, module: &Module, args: Vec<&Option<Value>>) -> RunResult {
        panic!("Not callable: {}", type_name::<Self>())
    }

    fn get_type(&self) -> Type;
}

#[derive(serde::Deserialize, serde::Serialize)]
struct RawValue(Type, String);

impl<'de> serde::Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let RawValue(typ, data) = RawValue::deserialize(deserializer)?;

        Ok(Value {
            data: deserialize_type(&typ, data),
            typ,
        })
    }
}

impl serde::Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let data = self.data.pre_serialize().expect("NOT SERIALIZABLE");

        let raw_value = RawValue(self.typ.clone(), ron::ser::to_string(data).unwrap());

        raw_value.serialize(serializer)
    }
}
pub trait PreSerialize {
    fn pre_serialize<'a>(&'a self) -> Option<&'a dyn erased_serde::Serialize>;
}

impl<T: erased_serde::Serialize> PreSerialize for T {
    fn pre_serialize<'a>(&'a self) -> Option<&'a dyn erased_serde::Serialize> {
        Some(self)
    }
}

impl<T: PartialEq + Clone + Debug + PrevalValue + 'static> ValueData for T {
    fn vclone(&self) -> Box<dyn ValueData> {
        let c = self.clone();
        Box::new(c)
    }

    fn veq(&self, other: &Value) -> bool {
        match other.data.as_any().downcast_ref::<T>() {
            Some(other) => self == other,
            None => panic!("Can't compare string to non-string"),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn index(&mut self, value: &Value) -> Value {
        self.vindex(value)
    }

    fn call(&mut self, module: &Module, args: Vec<&Option<Value>>) -> RunResult {
        self.vcall(module, args)
    }

    fn vto_string(&self) -> String {
        format!("{self:?}")
    }

    fn pre_serialize<'a>(&'a self) -> Option<&'a dyn erased_serde::Serialize> {
        PreSerialize::pre_serialize(self)
    }

    fn get_type(&self) -> Type {
        PrevalValue::get_type(self)
    }
}

// // TODO: Remove and replace with proper native functions when types are done
// #[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
// pub struct Print;
// impl PrevalValue for Print {
//     fn vcall(&mut self, module: &Module, args: Vec<&Option<Value>>) -> RunResult {
//         match [args[0], args[1]] {
//             [Some(_), Some(v)] => {
//                 println!("{v:?}");
//                 RunResult::Concrete(Value::new(EmptyTuple))
//             }
//             [Some(_), None] => panic!("IO is present but message is not"),
//             _ => RunResult::Residualise,
//         }
//     }

//     fn get_type(&self) -> Type {
//         Type::Print
//     }
// }
