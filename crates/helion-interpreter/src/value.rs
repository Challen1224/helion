use helion_ast::FunctionValue;
use crate::RuntimeError;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
    Function(FunctionValue),

    // Cannot derive PartialEq for function pointers
    NativeFunction(fn(Vec<Value>) -> Result<Value, RuntimeError>),

    Array(Vec<Value>),

    // ⭐ NEW: Object value
    Object(HashMap<String, Value>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::Array(a), Value::Array(b)) => a == b,

            // ⭐ Object deep equality
            (Value::Object(a), Value::Object(b)) => a == b,

            // Functions are never equal
            (Value::Function(_), Value::Function(_)) => false,
            (Value::NativeFunction(_), Value::NativeFunction(_)) => false,

            _ => false,
        }
    }
}