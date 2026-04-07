use std::{collections::HashMap, fmt};

use crate::errors::{InterpreterError, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum VariableValue {
    Str { value: String },
    Num { value: f64 },
    Boolean { value: bool },
    Nil,
}

impl VariableValue {
    pub fn is_truthy(&self) -> bool {
        !matches!(
            self,
            VariableValue::Boolean { value: false } | VariableValue::Nil
        )
    }
}

impl fmt::Display for VariableValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            VariableValue::Str { value } => write!(f, "{value}"),
            VariableValue::Num { value } => write!(f, "{value:}"),
            VariableValue::Boolean { value } => write!(f, "{value}"),
            VariableValue::Nil => write!(f, "nil"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Environment {
    values: HashMap<String, VariableValue>,
    enclosing: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn scope(self) -> Environment {
        Environment {
            values: HashMap::new(),
            enclosing: Some(Box::new(self)),
        }
    }

    pub fn unscope(self) -> Option<Box<Environment>> {
        self.enclosing
    }

    pub fn get(&self, name: &str) -> Result<&VariableValue> {
        match self.values.get(name) {
            Some(value) => Ok(value),
            None => match &self.enclosing {
                Some(env) => env.get(name),
                None => Err(InterpreterError::runtime_error(
                    1,
                    format!("Undefined variable '{name}'."),
                )),
            },
        }
    }

    pub fn define(&mut self, name: String, value: VariableValue) {
        self.values.insert(name, value);
    }

    pub fn assign(&mut self, name: String, value: VariableValue) -> Result<()> {
        if self.values.contains_key(&name) {
            self.values.insert(name, value);
            Ok(())
        } else {
            match &mut self.enclosing {
                Some(env) => env.assign(name, value),
                None => Err(InterpreterError::runtime_error(
                    1,
                    format!("Undefined variable '{name}'."),
                )),
            }
        }
    }
}
