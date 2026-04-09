use std::ffi::{CString, c_void};

use crate::value::{
    PrevalValue, Value,
    primitive::{EmptyTuple, IO},
    typ::{Signature, Type},
};
use libloading::Library;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NativeFunction {
    pub lib_name: String,
    pub func_name: String,
    pub signature: Signature,
}

impl PrevalValue for NativeFunction {
    fn get_type(&self) -> Type {
        Type::NativeFunction
    }

    fn vcall(
        &mut self,
        module: &crate::ir::Module,
        args: Vec<&Option<super::Value>>,
    ) -> crate::vm::RunResult {
        unsafe {
            let lib = Library::new(libloading::library_filename(self.lib_name.clone())).unwrap();

            let symbol = lib
                .get::<extern "C" fn(args: Vec<&Option<super::Value>>) -> crate::vm::RunResult>(
                    self.func_name.as_bytes(),
                )
                .unwrap();

            symbol(args);
            crate::vm::RunResult::Concrete(Value::new(EmptyTuple))
        }
    }
}
