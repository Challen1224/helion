use std::collections::HashMap;
use crate::value::Value;

#[derive(Debug, Clone)]
pub struct Env {
    parent: Option<Box<Env>>,
    vars: HashMap<String, Value>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            parent: None,
            vars: HashMap::new(),
        }
    }

    pub fn child(&self) -> Self {
        Self {
            parent: Some(Box::new(self.clone())),
            vars: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.vars.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(v) = self.vars.get(name) {
            return Some(v.clone());
        }
        if let Some(parent) = &self.parent {
            return parent.get(name);
        }
        None
    }
}