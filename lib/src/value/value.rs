use std::{
    any::{Any, type_name},
    fmt::Debug,
};

use crate::{
    typ::Program,
    value::runtime_type::{TypeDeserializer, deserialize_type},
    vm::RunResult,
};

#[repr(C)]
#[derive(Clone, Debug)]
pub struct Value {
    pub deserializer: TypeDeserializer,
    pub data: Box<dyn ValueData>,
    pub typ: usize,
}

impl Value {
    pub fn new<T: ValueData>(value: T, typ: usize) -> Value {
        Value {
            deserializer: value.get_deserializer(),
            data: value.vclone(),
            typ,
        }
    }
}

// impl Debug for Value {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         Debug::fmt(&self.data, f)
//     }
// }

pub trait ValueData: Debug {
    fn vclone(&self) -> Box<dyn ValueData>;
    fn index(&mut self, _value: &Value) -> Value {
        panic!("Type is not indexable")
    }
    fn call(&mut self, module: &mut Program, args: Vec<&Option<Value>>) -> RunResult;
    fn vto_string(&self) -> String;
    fn veq(&self, other: &Value) -> bool;
    fn as_any(&self) -> &dyn Any;
    fn pre_serialize<'a>(&'a self) -> Option<&'a dyn erased_serde::Serialize>;
    fn get_deserializer(&self) -> TypeDeserializer;
    fn should_poison(&self) -> bool;
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        self.data.veq(other)
    }
}

impl Eq for Value {}

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
    fn vindex(&mut self, _value: &Value) -> Value {
        panic!("Not indexable: {}", type_name::<Self>())
    }

    fn vcall(&mut self, _module: &mut Program, _args: Vec<&Option<Value>>) -> RunResult {
        panic!("Not callable: {}", type_name::<Self>())
    }

    fn vshould_poison(&self) -> bool {
        false
    }

    fn get_type(&self) -> TypeDeserializer;
}

#[derive(serde::Deserialize, serde::Serialize)]
struct RawValue(TypeDeserializer, String, usize);

impl<'de> serde::Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let RawValue(deserializer, data, typ) = RawValue::deserialize(deserializer)?;

        Ok(Value {
            data: deserialize_type(&deserializer, data),
            deserializer,
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

        let raw_value = RawValue(
            self.deserializer.clone(),
            ron::ser::to_string(data).unwrap(),
            self.typ,
        );

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

    fn call(&mut self, module: &mut Program, args: Vec<&Option<Value>>) -> RunResult {
        self.vcall(module, args)
    }

    fn vto_string(&self) -> String {
        format!("{self:?}")
    }

    fn pre_serialize<'a>(&'a self) -> Option<&'a dyn erased_serde::Serialize> {
        PreSerialize::pre_serialize(self)
    }

    fn get_deserializer(&self) -> TypeDeserializer {
        PrevalValue::get_type(self)
    }

    fn should_poison(&self) -> bool {
        self.vshould_poison()
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
