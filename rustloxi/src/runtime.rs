use std::fmt;

use crate::{
    ast::Stmt,
    errors::{InterpreterError, Result},
    interpreter::Interpreter,
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
        closure: usize, // Index of the captured env
    },
    NativeClock,
}

impl LoxCallable {
    pub fn call(
        &mut self,
        interpreter: &mut Interpreter,
        line: u32,
        args: &[VariableValue],
    ) -> Result<VariableValue> {
        match self {
            LoxCallable::Function {
                name: _,
                params,
                body,
                closure,
            } => {
                // TODO: add local scope pointing to closure scope. Interpreter needs a special API for that?
                interpreter.scope();
                for (p, a) in std::iter::zip(params, args) {
                    interpreter.define(*closure, p.to_string(), a.clone());
                }

                let result = match interpreter.execute_block(body) {
                    Ok(_) => Ok(VariableValue::Nil),
                    Err(InterpreterError::Return { value }) => Ok(value),
                    Err(err) => Err(err),
                };

                interpreter.unscope();

                result
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
            LoxCallable::Function { params, .. } => params.len(),
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
                raw: LoxCallable::Function { name, .. },
            } => write!(f, "<fn {name}>"),
            VariableValue::Function {
                raw: LoxCallable::NativeClock,
            } => write!(f, "<native fn>"),
        }
    }
}
