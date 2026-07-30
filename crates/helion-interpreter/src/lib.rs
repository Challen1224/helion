use helion_ast::{BinaryOp, Expr, FunctionValue, Program, Stmt};
use thiserror::Error;

mod env;
mod value;

use env::Env;
use value::Value;
use std::collections::HashMap;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Undefined variable '{0}'")]
    UndefinedVariable(String),

    #[error("Return outside of function")]
    ReturnOutsideFunction,

    #[error("Type error: {0}")]
    TypeError(String),

    #[error("Assertion failed: {0}")]
    AssertionFailed(String),
}

#[derive(Debug)]
enum ExecResult {
    Value(Value),
    Return(Value),
}

pub struct Interpreter;

impl Interpreter {
    pub fn run(&self, program: &Program) -> Result<Value, RuntimeError> {
        let mut env = Env::new();

        // print()
        env.define(
            "print".into(),
            Value::NativeFunction(|args| {
                for arg in args {
                    println!("{:?}", arg);
                }
                Ok(Value::Null)
            }),
        );

        // clock()
        env.define(
            "clock".into(),
            Value::NativeFunction(|_args| {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64();
                Ok(Value::Number(now))
            }),
        );

        // assert()
        env.define(
            "assert".into(),
            Value::NativeFunction(|args| {
                if args.len() != 1 {
                    return Err(RuntimeError::TypeError(
                        "assert() takes exactly one argument".into(),
                    ));
                }

                match &args[0] {
                    Value::Bool(true) => Ok(Value::Null),
                    Value::Bool(false) => Err(RuntimeError::AssertionFailed(
                        "expected true, got false".into(),
                    )),
                    other => Err(RuntimeError::TypeError(format!(
                        "assert() requires a boolean, got {:?}",
                        other
                    ))),
                }
            }),
        );

        // to_string()
        env.define(
            "to_string".into(),
            Value::NativeFunction(|args| {
                if args.len() != 1 {
                    return Err(RuntimeError::TypeError(
                        "to_string() takes exactly one argument".into(),
                    ));
                }

                let s = match &args[0] {
                    Value::Number(n) => n.to_string(),
                    Value::String(st) => st.clone(),
                    Value::Bool(b) => b.to_string(),
                    Value::Null => "null".into(),
                    Value::Function(_) => "<function>".into(),
                    Value::NativeFunction(_) => "<native function>".into(),
                    Value::Array(v) => {
                        let parts: Vec<String> =
                            v.iter().map(|x| format!("{:?}", x)).collect();
                        format!("[{}]", parts.join(", "))
                    }
                    Value::Object(map) => {
                        let mut parts = Vec::new();
                        for (k, v) in map {
                            parts.push(format!("{}: {:?}", k, v));
                        }
                        format!("{{{}}}", parts.join(", "))
                    }
                };

                Ok(Value::String(s))
            }),
        );

        // input()
        env.define(
            "input".into(),
            Value::NativeFunction(|_args| {
                use std::io::{self, Write};

                io::stdout().flush().unwrap();

                let mut buffer = String::new();
                io::stdin().read_line(&mut buffer).unwrap();

                Ok(Value::String(buffer.trim().to_string()))
            }),
        );

        // register user-defined functions
        for stmt in &program.stmts {
            if let Stmt::Function { name, params, body } = stmt {
                env.define(
                    name.clone(),
                    Value::Function(FunctionValue {
                        params: params.clone(),
                        body: body.clone(),
                    }),
                );
            }
        }

        // main()
        let main_val = env
            .get("main")
            .ok_or_else(|| RuntimeError::UndefinedVariable("main".into()))?;

        match main_val {
            Value::Function(f) => self.call_function(&mut env, &f, Vec::new()),
            _ => Err(RuntimeError::TypeError("main is not a function".into())),
        }
    }

    fn call_function(
        &self,
        env: &mut Env,
        func: &FunctionValue,
        args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        if args.len() != func.params.len() {
            return Err(RuntimeError::TypeError("argument count mismatch".into()));
        }

        let mut local = env.child();

        for (name, value) in func.params.iter().zip(args.into_iter()) {
            local.define(name.clone(), value);
        }

        match self.exec_stmt(&mut local, &*func.body)? {
            ExecResult::Return(v) => Ok(v),
            ExecResult::Value(v) => Ok(v),
        }
    }

    fn exec_stmt(&self, env: &mut Env, stmt: &Stmt) -> Result<ExecResult, RuntimeError> {
        match stmt {
            Stmt::Let { name, value } => {
                let v = self.eval_expr(env, value)?;
                env.define(name.clone(), v);
                Ok(ExecResult::Value(Value::Null))
            }

            Stmt::Assign { name, value } => {
                let v = self.eval_expr(env, value)?;
                if !env.set(name, v) {
                    return Err(RuntimeError::UndefinedVariable(name.clone()));
                }
                Ok(ExecResult::Value(Value::Null))
            }

            // ⭐ ARRAY ASSIGNMENT
            Stmt::ArrayAssign { array, index, value } => {
                let arr_name = match array {
                    Expr::Ident(ref name) => name.clone(),
                    _ => {
                        return Err(RuntimeError::TypeError(
                            "Left side of array assignment must be an identifier".into(),
                        ))
                    }
                };

                let arr_val = env
                    .get(&arr_name)
                    .ok_or_else(|| RuntimeError::UndefinedVariable(arr_name.clone()))?;

                let idx_val = self.eval_expr(env, index)?;
                let new_val = self.eval_expr(env, value)?;

                let idx = match idx_val {
                    Value::Number(n) => n as usize,
                    _ => {
                        return Err(RuntimeError::TypeError(
                            "array index must be a number".into(),
                        ))
                    }
                };

                match arr_val {
                    Value::Array(mut vec) => {
                        if idx >= vec.len() {
                            return Err(RuntimeError::TypeError(
                                "array index out of bounds".into(),
                            ));
                        }

                        vec[idx] = new_val;
                        env.set(&arr_name, Value::Array(vec));

                        Ok(ExecResult::Value(Value::Null))
                    }
                    _ => Err(RuntimeError::TypeError("indexing non-array".into())),
                }
            }

            // ⭐ OBJECT PROPERTY ASSIGNMENT
            Stmt::ObjectAssign {
                object,
                property,
                value,
            } => {
                let obj_name = match object {
                    Expr::Ident(ref name) => name.clone(),
                    _ => {
                        return Err(RuntimeError::TypeError(
                            "Left side of object assignment must be an identifier".into(),
                        ))
                    }
                };

                let obj_val = env
                    .get(&obj_name)
                    .ok_or_else(|| RuntimeError::UndefinedVariable(obj_name.clone()))?;

                let new_val = self.eval_expr(env, value)?;

                match obj_val {
                    Value::Object(mut map) => {
                        map.insert(property.clone(), new_val);
                        env.set(&obj_name, Value::Object(map));
                        Ok(ExecResult::Value(Value::Null))
                    }
                    _ => Err(RuntimeError::TypeError("property access on non-object".into())),
                }
            }

            Stmt::Return { value } => {
                let v = self.eval_expr(env, value)?;
                Ok(ExecResult::Return(v))
            }

            Stmt::ExprStmt { expr } => {
                let v = self.eval_expr(env, expr)?;
                Ok(ExecResult::Value(v))
            }

            Stmt::While { condition, body } => {
                loop {
                    let cond = self.eval_expr(env, condition)?;

                    match cond {
                        Value::Bool(true) => match self.exec_stmt(env, body)? {
                            ExecResult::Return(v) => return Ok(ExecResult::Return(v)),
                            ExecResult::Value(_) => {}
                        },
                        Value::Bool(false) => break,
                        _ => {
                            return Err(RuntimeError::TypeError(
                                "while condition must be boolean".into(),
                            ))
                        }
                    }
                }

                Ok(ExecResult::Value(Value::Null))
            }

            Stmt::Block { stmts } => {
                let mut last = Value::Null;

                for stmt in stmts {
                    match self.exec_stmt(env, stmt)? {
                        ExecResult::Return(v) => return Ok(ExecResult::Return(v)),
                        ExecResult::Value(v) => last = v,
                    }
                }

                Ok(ExecResult::Value(last))
            }

            Stmt::Function { .. } => Ok(ExecResult::Value(Value::Null)),

            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond = self.eval_expr(env, condition)?;

                match cond {
                    Value::Bool(true) => self.exec_stmt(env, then_branch),
                    Value::Bool(false) => {
                        if let Some(else_branch) = else_branch {
                            self.exec_stmt(env, else_branch)
                        } else {
                            Ok(ExecResult::Value(Value::Null))
                        }
                    }
                    _ => Err(RuntimeError::TypeError(
                        "if condition must be boolean".into(),
                    )),
                }
            }
        }
    }

    fn eval_expr(&self, env: &Env, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Ident(name) => {
                if name == "true" {
                    return Ok(Value::Bool(true));
                }
                if name == "false" {
                    return Ok(Value::Bool(false));
                }

                env.get(name)
                    .ok_or_else(|| RuntimeError::UndefinedVariable(name.clone()))
            }

            Expr::Number(n) => Ok(Value::Number(*n)),
            Expr::String(s) => Ok(Value::String(s.clone())),

            Expr::UnaryMinus(inner) => {
                let v = self.eval_expr(env, inner)?;
                match v {
                    Value::Number(n) => Ok(Value::Number(-n)),
                    _ => Err(RuntimeError::TypeError("Unary minus on non-number".into())),
                }
            }

            Expr::UnaryBang(inner) => {
                let v = self.eval_expr(env, inner)?;
                match v {
                    Value::Bool(b) => Ok(Value::Bool(!b)),
                    _ => Err(RuntimeError::TypeError("Unary ! on non-bool".into())),
                }
            }

            Expr::Binary { left, op, right } => {
                let l = self.eval_expr(env, left)?;
                let r = self.eval_expr(env, right)?;
                self.eval_binary(l, *op, r)
            }

            Expr::Call { callee, args } => {
                let callee_val = self.eval_expr(env, callee)?;
                let mut arg_vals = Vec::new();
                for a in args {
                    arg_vals.push(self.eval_expr(env, a)?);
                }

                match callee_val {
                    Value::Function(f) => {
                        let mut env_clone = env.clone();
                        self.call_function(&mut env_clone, &f, arg_vals)
                    }

                    Value::NativeFunction(func) => func(arg_vals),

                    _ => Err(RuntimeError::TypeError("call on non-function".into())),
                }
            }

            // ⭐ OBJECT LITERAL
            Expr::Object(fields) => {
                let mut map = HashMap::new();
                for (key, expr) in fields {
                    let val = self.eval_expr(env, expr)?;
                    map.insert(key.clone(), val);
                }
                Ok(Value::Object(map))
            }

            // ⭐ PROPERTY ACCESS
            Expr::Property { object, property } => {
                let obj_val = self.eval_expr(env, object)?;

                match obj_val {
                    Value::Object(map) => map
                        .get(property)
                        .cloned()
                        .ok_or_else(|| RuntimeError::TypeError(format!(
                            "property '{}' not found",
                            property
                        ))),
                    _ => Err(RuntimeError::TypeError("property access on non-object".into())),
                }
            }

            // Array literal
            Expr::Array(items) => {
                let mut vals = Vec::new();
                for item in items {
                    vals.push(self.eval_expr(env, item)?);
                }
                Ok(Value::Array(vals))
            }

            // Indexing: array[index]
            Expr::Index { array, index } => {
                let arr_val = self.eval_expr(env, array)?;
                let idx_val = self.eval_expr(env, index)?;

                let idx = match idx_val {
                    Value::Number(n) => n as usize,
                    _ => {
                        return Err(RuntimeError::TypeError(
                            "array index must be a number".into(),
                        ))
                    }
                };

                match arr_val {
                    Value::Array(vec) => vec
                        .get(idx)
                        .cloned()
                        .ok_or_else(|| RuntimeError::TypeError("array index out of bounds".into())),
                    _ => Err(RuntimeError::TypeError("indexing non-array".into())),
                }
            }
        }
    }

    fn eval_binary(&self, l: Value, op: BinaryOp, r: Value) -> Result<Value, RuntimeError> {
        match op {
            BinaryOp::Plus => match (l, r) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                (Value::String(a), Value::String(b)) => Ok(Value::String(a + &b)),
                _ => Err(RuntimeError::TypeError("Invalid + operands".into())),
            },

            BinaryOp::Minus => match (l, r) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
                _ => Err(RuntimeError::TypeError("Invalid - operands".into())),
            },

            BinaryOp::Star => match (l, r) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
                _ => Err(RuntimeError::TypeError("Invalid * operands".into())),
            },

            BinaryOp::Slash => match (l, r) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a / b)),
                _ => Err(RuntimeError::TypeError("Invalid / operands".into())),
            },

            BinaryOp::Less => Self::cmp(l, r, |a, b| a < b),
            BinaryOp::LessEqual => Self::cmp(l, r, |a, b| a <= b),
            BinaryOp::Greater => Self::cmp(l, r, |a, b| a > b),
            BinaryOp::GreaterEqual => Self::cmp(l, r, |a, b| a >= b),

            BinaryOp::EqualEqual => Ok(Value::Bool(Self::eq(&l, &r))),
            BinaryOp::BangEqual => Ok(Value::Bool(!Self::eq(&l, &r))),
        }
    }

    fn cmp<F: Fn(f64, f64) -> bool>(l: Value, r: Value, f: F) -> Result<Value, RuntimeError> {
        match (l, r) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(f(a, b))),
            _ => Err(RuntimeError::TypeError("Comparison on non-numbers".into())),
        }
    }

    fn eq(a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Number(x), Value::Number(y)) => x == y,
            (Value::String(x), Value::String(y)) => x == y,
            (Value::Bool(x), Value::Bool(y)) => x == y,
            (Value::Null, Value::Null) => true,
            (Value::Array(x), Value::Array(y)) => x == y,
            (Value::Object(x), Value::Object(y)) => x == y,
            _ => false,
        }
    }
}