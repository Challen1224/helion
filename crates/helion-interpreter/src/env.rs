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

    // FIX: child must reference the SAME environment, not a clone
    pub fn child(&mut self) -> Self {
        Self {
            parent: Some(Box::new(self.clone())),
            vars: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        // FIX: assign to existing variable in parent if it exists
        if self.vars.contains_key(&name) {
            self.vars.insert(name, value);
            return;
        }

        if let Some(parent) = &mut self.parent {
            if parent.vars.contains_key(&name) {
                parent.vars.insert(name.clone(), value);
                return;
            }
        }

        // Otherwise define locally
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