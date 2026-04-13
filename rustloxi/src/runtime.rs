use std::{collections::HashMap, fmt};

use crate::{
    ast::Stmt,
    errors::{InterpreterError, Result},
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
        params: Vec<(u32, String)>,
        body: Vec<Stmt>,
        closure: usize, // Index of the captured env
    },
    NativeClock,
}

impl LoxCallable {
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

// No field for tracking freed indices, because some left scopes could be captured into closures
#[derive(Debug)]
pub struct Environment {
    scopes: Vec<Scope>,
}

#[derive(Debug)]
struct Scope {
    data: HashMap<String, VariableValue>,
    parent: Option<usize>,
}

impl Environment {
    pub fn new() -> Self {
        let mut globals = HashMap::new();
        globals.insert(String::from("clock"), CLOCK);

        let global_scope = Scope {
            data: globals,
            parent: None,
        };

        Environment {
            scopes: vec![global_scope],
        }
    }

    pub fn get(&self, scope_idx: usize, name: &str, line: u32) -> Result<&VariableValue> {
        let mut scope_idx = scope_idx;

        loop {
            let scope = &self.scopes[scope_idx];
            match scope.data.get(name) {
                Some(value) => break Ok(value),
                None => match scope.parent {
                    Some(parent_idx) => {
                        scope_idx = parent_idx;
                    }
                    None => {
                        break Err(InterpreterError::undefined_variable(line, name.to_string()));
                    }
                },
            }
        }
    }

    pub fn define(&mut self, scope_idx: usize, name: String, value: VariableValue) {
        self.scopes[scope_idx].data.insert(name, value);
    }

    pub fn assign(
        &mut self,
        scope_idx: usize,
        name: String,
        value: VariableValue,
        line: u32,
    ) -> Result<()> {
        let mut scope_idx = scope_idx;

        loop {
            let scope = &mut self.scopes[scope_idx];
            if scope.data.contains_key(name.as_str()) {
                scope.data.insert(name, value);
                break Ok(());
            } else {
                match scope.parent {
                    Some(parent_idx) => {
                        scope_idx = parent_idx;
                    }
                    None => {
                        break Err(InterpreterError::undefined_variable(line, name));
                    }
                }
            }
        }
    }

    pub fn scope(&mut self, parent_scope_idx: usize) -> usize {
        let scope = Scope {
            data: HashMap::new(),
            parent: Some(parent_scope_idx),
        };

        self.scopes.push(scope);
        self.scopes.len() - 1
    }

    pub fn unscope(&mut self, scope_idx: usize) -> usize {
        self.scopes[scope_idx].parent.unwrap_or(0)
    }

    pub fn get_at(
        &self,
        scope_idx: usize,
        name: &str,
        distance: usize,
        line: u32,
    ) -> Result<&VariableValue> {
        let mut ancestor = Some(&self.scopes[scope_idx]);
        for _ in 0..distance {
            ancestor = ancestor.and_then(|s| s.parent.map(|p_idx| &self.scopes[p_idx]));
        }

        ancestor
            .and_then(|s| s.data.get(name))
            .ok_or_else(|| InterpreterError::undefined_variable(line, name.to_string()))
    }

    pub fn assign_at(
        &mut self,
        scope_idx: usize,
        name: String,
        value: VariableValue,
        distance: usize,
        line: u32,
    ) -> Result<()> {
        let mut ancestor = Some(&mut self.scopes[scope_idx]);
        for _ in 0..distance {
            match ancestor.and_then(|s| s.parent) {
                Some(parent) => {
                    ancestor = Some(&mut self.scopes[parent]);
                }
                None => return Err(InterpreterError::undefined_variable(line, name)),
            }
        }

        match ancestor {
            Some(anc) => {
                anc.data.insert(name, value);
                Ok(())
            }
            None => Err(InterpreterError::undefined_variable(line, name)),
        }
    }
}
