use crate::value::primitive::{EmptyTuple, IO};
use crate::value::typ::Type;
use crate::value::{PreSerialize, PrevalValue, Value};
use crate::vm::RunResult;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    name: String,
    #[serde(skip)]
    library: Option<Arc<Mutex<libloading::Library>>>,
}

impl PartialEq for Library {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl PrevalValue for Library {
    fn get_type(&self) -> Type {
        Type::Library
    }

    fn vindex(&mut self, value: &super::Value) -> super::Value {
        let name = if let Some(name) = value.data.as_any().downcast_ref::<String>() {
            name.clone()
        } else {
            panic!("Library can only be indexed by string");
        };

        let library = if let Some(library) = &self.library {
            library.clone()
        } else {
            let a = Arc::new(Mutex::new(unsafe {
                libloading::Library::new(self.name.clone()).unwrap()
            }));
            self.library = Some(a.clone());
            a
        };

        Value::new(Symbol {
            name,
            library_name: self.name.clone(),
            library: Some(library.clone()),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    #[serde(skip)]
    library: Option<Arc<Mutex<libloading::Library>>>,
    name: String,
    library_name: String,
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.library_name == other.library_name
    }
}

impl PrevalValue for Symbol {
    fn get_type(&self) -> Type {
        Type::Symbol
    }

    fn vcall(
        &mut self,
        module: &crate::ir::Module,
        args: Vec<&Option<Value>>,
    ) -> crate::vm::RunResult {
        if let Some(Some(arg)) = args.first() {
            if let Some(_) = arg.data.as_any().downcast_ref::<IO>() {
            } else {
                panic!("NON-IO in symbol call")
            }
        } else {
            return RunResult::Residualise;
        }

        let library = if let Some(library) = &self.library {
            library.clone()
        } else {
            let a = Arc::new(Mutex::new(unsafe {
                libloading::Library::new(self.name.clone()).unwrap()
            }));
            self.library = Some(a.clone());
            a
        };

        unsafe {
            let library = library.as_ref().lock().unwrap();
            let symbol = library.get::<fn()>(self.name.as_bytes()).unwrap();
            symbol();
        }

        crate::vm::RunResult::Concrete(Value::new(EmptyTuple))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LibraryConstructor;

impl PrevalValue for LibraryConstructor {
    fn get_type(&self) -> Type {
        Type::LibraryConstructor
    }

    fn vcall(
        &mut self,
        module: &crate::ir::Module,
        args: Vec<&Option<Value>>,
    ) -> crate::vm::RunResult {
        if let Some(Some(name)) = args.first() {
            if let Some(name) = name.data.as_any().downcast_ref::<String>() {
                RunResult::Concrete(Value::new(Library {
                    name: name.clone(),
                    library: None,
                }))
            } else {
                RunResult::Residualise
            }
        } else {
            panic!("Arg1 missing library constructor")
        }
    }
}
