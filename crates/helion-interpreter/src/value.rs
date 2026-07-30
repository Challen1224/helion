use helion_ast::FunctionValue;

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
    Function(FunctionValue),
}