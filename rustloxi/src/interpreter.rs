use crate::errors::{InterpreterError, Result};
use rustloxi::VariableValue;

use crate::ast::Expr;

pub fn interpret(expr: &Expr) -> Result<VariableValue> {
    evaluate(expr).inspect(|v| println!("{v}"))
}

// FIX: fix to include actual line number instead of 1
fn evaluate(expr: &Expr) -> Result<VariableValue> {
    match expr {
        Expr::Grouping { expr } => evaluate(expr),
        Expr::Literal { value } => Ok(value.clone()),
        Expr::Nil => Ok(VariableValue::Nil),
        Expr::Negative { expr } => match evaluate(expr)? {
            VariableValue::Num { value } => Ok(VariableValue::Num { value: -value }),
            _ => Err(InterpreterError::runtime_error(
                1,
                String::from("Operand must be a number."),
            )),
        },
        Expr::Not { expr } => {
            let result = evaluate(expr)?;
            Ok(VariableValue::Boolean {
                value: !result.is_truthy(),
            })
        }
        Expr::Equals { left, right } => Ok(VariableValue::Boolean {
            value: evaluate(left)? == evaluate(right)?,
        }),
        Expr::NotEquals { left, right } => Ok(VariableValue::Boolean {
            value: evaluate(left)? != evaluate(right)?,
        }),
        Expr::Less { left, right } => match (evaluate(left)?, evaluate(right)?) {
            (VariableValue::Num { value: left }, VariableValue::Num { value: right }) => {
                Ok(VariableValue::Boolean {
                    value: left < right,
                })
            }
            _ => Err(InterpreterError::operands_must_be_numbers_error(1)),
        },
        Expr::LessEquals { left, right } => match (evaluate(left)?, evaluate(right)?) {
            (VariableValue::Num { value: left }, VariableValue::Num { value: right }) => {
                Ok(VariableValue::Boolean {
                    value: left <= right,
                })
            }
            _ => Err(InterpreterError::operands_must_be_numbers_error(1)),
        },
        Expr::Greater { left, right } => match (evaluate(left)?, evaluate(right)?) {
            (VariableValue::Num { value: left }, VariableValue::Num { value: right }) => {
                Ok(VariableValue::Boolean {
                    value: left > right,
                })
            }
            _ => Err(InterpreterError::operands_must_be_numbers_error(1)),
        },
        Expr::GreaterEquals { left, right } => match (evaluate(left)?, evaluate(right)?) {
            (VariableValue::Num { value: left }, VariableValue::Num { value: right }) => {
                Ok(VariableValue::Boolean {
                    value: left >= right,
                })
            }
            _ => Err(InterpreterError::operands_must_be_numbers_error(1)),
        },
        Expr::Plus { left, right } => match (evaluate(left)?, evaluate(right)?) {
            (VariableValue::Num { value: left }, VariableValue::Num { value: right }) => {
                Ok(VariableValue::Num {
                    value: left + right,
                })
            }
            (VariableValue::Str { value: left }, VariableValue::Str { value: right }) => {
                Ok(VariableValue::Str {
                    value: format!("{left}{right}"),
                })
            }
            _ => Err(InterpreterError::runtime_error(
                1,
                String::from("Operands must be two numbers or two strings."),
            )),
        },
        Expr::Minus { left, right } => match (evaluate(left)?, evaluate(right)?) {
            (VariableValue::Num { value: left }, VariableValue::Num { value: right }) => {
                Ok(VariableValue::Num {
                    value: left - right,
                })
            }
            _ => Err(InterpreterError::operands_must_be_numbers_error(1)),
        },
        Expr::Multiply { left, right } => match (evaluate(left)?, evaluate(right)?) {
            (VariableValue::Num { value: left }, VariableValue::Num { value: right }) => {
                Ok(VariableValue::Num {
                    value: left * right,
                })
            }
            _ => Err(InterpreterError::operands_must_be_numbers_error(1)),
        },
        Expr::Divide { left, right } => match (evaluate(left)?, evaluate(right)?) {
            (VariableValue::Num { value: left }, VariableValue::Num { value: right }) => {
                Ok(VariableValue::Num {
                    value: left / right,
                })
            }
            _ => Err(InterpreterError::operands_must_be_numbers_error(1)),
        },
    }
}
