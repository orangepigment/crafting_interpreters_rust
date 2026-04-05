use std::fmt;

// TODO: think where to store implementation models
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
