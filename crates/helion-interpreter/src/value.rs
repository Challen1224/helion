use helion_ast::FunctionValue;
use crate::RuntimeError;

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
    Function(FunctionValue),

    NativeFunction(fn(Vec<Value>) -> Result<Value, RuntimeError>),
}