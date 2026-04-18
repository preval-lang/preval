use std::marker::PhantomData;

#[repr(C)]
pub struct RawAPI<Value> {
    pub drop_value: unsafe extern "C" fn(*mut Value),
    pub string_value_length: unsafe extern "C" fn(*const Value) -> usize,
    pub string_value_start: unsafe extern "C" fn(*const Value) -> *const u8,
    pub new_tuple_value: unsafe extern "C" fn() -> *mut Value,
    pub new_string_value: unsafe extern "C" fn(*const u8, usize) -> *mut Value,
}

pub struct Value {
    p: PhantomData<()>,
}

pub type API = RawAPI<Value>;
