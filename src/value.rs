use std::{
    any::{Any, type_name},
    fmt::Debug,
};

use serde::{
    Deserialize, Serialize,
    de::Visitor,
    ser::{SerializeMap, SerializeSeq, SerializeStruct},
};

use crate::typ::deserialize_type;

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

#[derive(Clone)]
pub struct Value {
    pub typ: String,
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
    fn index(&self, value: &Value) -> Value {
        panic!("Type is not indexable")
    }
    fn vto_string(&self) -> String;
    fn veq(&self, other: &Value) -> bool;
    fn as_any(&self) -> &dyn Any;
    fn pre_serialize<'a>(&'a self) -> Option<&'a dyn erased_serde::Serialize>;
    fn get_type(&self) -> String;
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

trait PrevalValue: PreSerialize {
    fn vindex(&self, value: &Value) -> Value {
        panic!("Not indexable: {}", type_name::<Self>())
    }

    fn get_type(&self) -> String;
}

#[derive(serde::Deserialize)]
struct RawValue(String, serde_value::Value);

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
        use serde::ser::SerializeTuple;

        let mut tup = serializer.serialize_tuple(2)?;
        tup.serialize_element(&self.typ)?;

        let data = self.data.pre_serialize();

        tup.serialize_element(&data)?;
        tup.end()
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

    fn index(&self, value: &Value) -> Value {
        self.vindex(value)
    }

    fn vto_string(&self) -> String {
        format!("{self:?}")
    }

    fn pre_serialize<'a>(&'a self) -> Option<&'a dyn erased_serde::Serialize> {
        PreSerialize::pre_serialize(self)
    }

    fn get_type(&self) -> String {
        PrevalValue::get_type(self)
    }
}

impl PrevalValue for String {
    fn vindex(&self, value: &Value) -> Value {
        match value.data.as_any().downcast_ref::<usize>() {
            Some(other) => Value::new(self.chars().nth(*other).unwrap().to_string()),
            None => panic!("Index string with non-usize"),
        }
    }

    fn get_type(&self) -> String {
        "String".to_string()
    }
}

impl PrevalValue for usize {
    fn get_type(&self) -> String {
        "usize".to_string()
    }
}

impl PrevalValue for bool {
    fn get_type(&self) -> String {
        "bool".to_string()
    }
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct IO;
impl PrevalValue for IO {
    fn get_type(&self) -> String {
        "IO".to_string()
    }
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct EmptyTuple;
impl PrevalValue for EmptyTuple {
    fn get_type(&self) -> String {
        "EmptyTuple".to_string()
    }
}
