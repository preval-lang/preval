use crate::{
	typ::{Type, type_id},
	value::{PrevalValue, Value, primitive::EmptyTuple, runtime_type::TypeDeserializer},
};
use libloading::Library;
use preval_api::RawAPI;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NativeFunction {
	pub lib_name: String,
	pub func_name: String,
}

impl PrevalValue for NativeFunction {
	fn get_type(&self) -> TypeDeserializer {
		TypeDeserializer::NativeFunction
	}

	fn vcall(
		&mut self,
		_module: &mut Vec<Type>,
		args: Vec<&Option<super::Value>>,
	) -> crate::vm::RunResult {
		unsafe {
			let lib = Library::new(libloading::library_filename(self.lib_name.clone())).unwrap();

			let symbol = lib
				.get::<extern "C" fn(
					api: *const RawAPI<Value>,
					argc: usize,
					args: *const *const Value,
				) -> *mut Value>(self.func_name.as_bytes())
				.unwrap();

			let mut args_c = Vec::new();
			for arg in args {
				match arg {
					Some(value) => args_c.push(value as *const Value),
					None => args_c.push(std::ptr::null_mut()),
				}
			}
			let args_ptr = args_c.as_ptr();
			let argc = args_c.len();

			let api = RawAPI::<Value> {
				drop_value,
				string_value_length,
				string_value_start,
				new_tuple_value,
				new_string_value,
			};

			match symbol(&api, argc, args_ptr) {
				p if std::ptr::null() == p => crate::vm::RunResult::Residualise,
				ptr => crate::vm::RunResult::Concrete(*Box::from_raw(ptr)),
			}
		}
	}
}

extern "C" fn drop_value(value: *mut Value) {
	unsafe {
		drop(Box::from_raw(value));
	}
}

extern "C" fn string_value_length(value: *const Value) -> usize {
	unsafe {
		match (*value).data.as_any().downcast_ref::<String>() {
			Some(s) => s.len(),
			None => panic!("Value is not a string"),
		}
	}
}

extern "C" fn string_value_start(value: *const Value) -> *const u8 {
	unsafe {
		match (*value).data.as_any().downcast_ref::<String>() {
			Some(s) => s.as_ptr(),
			None => panic!("Value is not a string"),
		}
	}
}

extern "C" fn new_tuple_value() -> *mut Value {
	Box::into_raw(Box::new(Value::new(EmptyTuple, type_id::empty_tuple)))
}

extern "C" fn new_string_value(value: *const u8, len: usize) -> *mut Value {
	Box::into_raw(Box::new(Value::new(
		String::from_utf8_lossy(unsafe { std::slice::from_raw_parts(value, len) }).into_owned(),
		type_id::String,
	)))
}
