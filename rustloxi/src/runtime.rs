use std::{collections::HashMap, fmt};

use crate::{
    ast::Stmt,
    errors::{InterpreterError, Result},
    interpreter::execute_block,
};

pub const CLOCK: VariableValue = VariableValue::Function {
    raw: LoxCallable::NativeClock,
};

#[derive(Debug, Clone, PartialEq)]
pub enum VariableValue {
    Str { value: String },
    Num { value: f64 },
    Boolean { value: bool },
    Nil,
    Function { raw: LoxCallable },
}

impl VariableValue {
    pub fn is_truthy(&self) -> bool {
        !matches!(
            self,
            VariableValue::Boolean { value: false } | VariableValue::Nil
        )
    }

    pub fn into_function(self, line: u32) -> Result<LoxCallable> {
        match self {
            VariableValue::Function { raw } => Ok(raw),
            _ => Err(InterpreterError::runtime_error(
                line,
                String::from("Can only call functions and classes."),
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoxCallable {
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    NativeClock,
}

impl LoxCallable {
    pub fn call(&self, line: u32, args: &[VariableValue]) -> Result<VariableValue> {
        match self {
            LoxCallable::Function {
                name: _,
                params,
                body,
            } => {
                // Set global env with clock and other global objects
                let mut env = Environment::new();
                env.define(String::from("clock"), CLOCK);
                for (p, a) in std::iter::zip(params, args) {
                    env.define(p.to_string(), a.clone());
                }

                match execute_block(body, env) {
                    Ok(_) => Ok(VariableValue::Nil),
                    Err(InterpreterError::Return { value }) => Ok(value),
                    Err(err) => Err(err),
                }
            }
            LoxCallable::NativeClock => {
                let now = std::time::SystemTime::now();
                let seconds = now
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("SystemTime before UNIX EPOCH!")
                    .as_secs_f64();

                Ok(VariableValue::Num { value: seconds })
            }
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            LoxCallable::Function {
                name: _,
                params,
                body: _,
            } => params.len(),
            LoxCallable::NativeClock => 0,
        }
    }
}

impl fmt::Display for VariableValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            VariableValue::Str { value } => write!(f, "{value}"),
            VariableValue::Num { value } => write!(f, "{value:}"),
            VariableValue::Boolean { value } => write!(f, "{value}"),
            VariableValue::Nil => write!(f, "nil"),
            VariableValue::Function {
                raw:
                    LoxCallable::Function {
                        name,
                        params: _,
                        body: _,
                    },
            } => write!(f, "<fn {name}>"),
            VariableValue::Function {
                raw: LoxCallable::NativeClock,
            } => write!(f, "<native fn>"),
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

    pub fn unscope(self) -> Environment {
        match self.enclosing {
            Some(outer_scope) => *outer_scope,
            None => self,
        }
    }

    pub fn get(&self, name: &str, line: u32) -> Result<&VariableValue> {
        match self.values.get(name) {
            Some(value) => Ok(value),
            None => match &self.enclosing {
                Some(env) => env.get(name, line),
                None => Err(InterpreterError::runtime_error(
                    line,
                    format!("Undefined variable '{name}'."),
                )),
            },
        }
    }

    pub fn define(&mut self, name: String, value: VariableValue) {
        self.values.insert(name, value);
    }

    pub fn assign(&mut self, name: String, value: VariableValue, line: u32) -> Result<()> {
        if self.values.contains_key(&name) {
            self.values.insert(name, value);
            Ok(())
        } else {
            match &mut self.enclosing {
                Some(env) => env.assign(name, value, line),
                None => Err(InterpreterError::runtime_error(
                    line,
                    format!("Undefined variable '{name}'."),
                )),
            }
        }
    }
}
