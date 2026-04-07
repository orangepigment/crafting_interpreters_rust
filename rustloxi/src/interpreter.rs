use crate::errors::{InterpreterError, Result};

use crate::{
    ast::{Expr, Stmt},
    state::{Environment, VariableValue},
};

pub fn interpret(stmts: &Vec<Stmt>) -> Result<()> {
    //evaluate_expr(expr, env).inspect(|v| println!("{v}"))
    let mut env = Environment::new();

    for s in stmts {
        env = execute_stmt(s, env)?;
    }

    Ok(())
}

fn execute_stmt(stmt: &Stmt, mut env: Environment) -> Result<Environment> {
    match stmt {
        Stmt::Expr { expr } => evaluate_expr(expr, &mut env).map(|_| env),
        Stmt::Print { expr } => {
            let value = evaluate_expr(expr, &mut env)?;
            println!("{value}");

            Ok(env)
        }
        Stmt::Var { name, initializer } => {
            let value = match initializer {
                Some(i) => evaluate_expr(i, &mut env)?,
                None => VariableValue::Nil,
            };

            env.define(name.clone(), value);

            Ok(env)
        }
        // TODO: swithc to RefCell for handling env?
        Stmt::Block { statements } => execute_block(statements, env.scope()),
    }
}

fn execute_block(statements: &[Stmt], env: Environment) -> Result<Environment> {
    let mut env = env;

    for stmt in statements {
        env = execute_stmt(stmt, env)?;
    }

    env.unscope()
        .map(|b| *b)
        // In practice this error should never happen
        .ok_or(InterpreterError::runtime_error(
            1,
            String::from("No paren environment in local scope"),
        ))
}

// FIX: fix to include actual line number instead of 1
fn evaluate_expr(expr: &Expr, env: &mut Environment) -> Result<VariableValue> {
    match expr {
        Expr::Grouping { expr } => evaluate_expr(expr, env),
        Expr::Literal { value } => Ok(value.clone()),
        Expr::Nil => Ok(VariableValue::Nil),
        Expr::Variable { name } => env.get(name).cloned(),
        Expr::Assignment { name, value } => {
            let value = evaluate_expr(value, env)?;
            env.assign(name.clone(), value.clone())?;

            Ok(value)
        }
        Expr::Negative { expr } => match evaluate_expr(expr, env)? {
            VariableValue::Num { value } => Ok(VariableValue::Num { value: -value }),
            _ => Err(InterpreterError::runtime_error(
                1,
                String::from("Operand must be a number."),
            )),
        },
        Expr::Not { expr } => {
            let result = evaluate_expr(expr, env)?;
            Ok(VariableValue::Boolean {
                value: !result.is_truthy(),
            })
        }
        Expr::Equals { left, right } => Ok(VariableValue::Boolean {
            value: evaluate_expr(left, env)? == evaluate_expr(right, env)?,
        }),
        Expr::NotEquals { left, right } => Ok(VariableValue::Boolean {
            value: evaluate_expr(left, env)? != evaluate_expr(right, env)?,
        }),
        Expr::Less { left, right } => match (evaluate_expr(left, env)?, evaluate_expr(right, env)?)
        {
            (VariableValue::Num { value: left }, VariableValue::Num { value: right }) => {
                Ok(VariableValue::Boolean {
                    value: left < right,
                })
            }
            _ => Err(InterpreterError::operands_must_be_numbers_error(1)),
        },
        Expr::LessEquals { left, right } => {
            match (evaluate_expr(left, env)?, evaluate_expr(right, env)?) {
                (VariableValue::Num { value: left }, VariableValue::Num { value: right }) => {
                    Ok(VariableValue::Boolean {
                        value: left <= right,
                    })
                }
                _ => Err(InterpreterError::operands_must_be_numbers_error(1)),
            }
        }
        Expr::Greater { left, right } => {
            match (evaluate_expr(left, env)?, evaluate_expr(right, env)?) {
                (VariableValue::Num { value: left }, VariableValue::Num { value: right }) => {
                    Ok(VariableValue::Boolean {
                        value: left > right,
                    })
                }
                _ => Err(InterpreterError::operands_must_be_numbers_error(1)),
            }
        }
        Expr::GreaterEquals { left, right } => {
            match (evaluate_expr(left, env)?, evaluate_expr(right, env)?) {
                (VariableValue::Num { value: left }, VariableValue::Num { value: right }) => {
                    Ok(VariableValue::Boolean {
                        value: left >= right,
                    })
                }
                _ => Err(InterpreterError::operands_must_be_numbers_error(1)),
            }
        }
        Expr::Plus { left, right } => match (evaluate_expr(left, env)?, evaluate_expr(right, env)?)
        {
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
        Expr::Minus { left, right } => {
            match (evaluate_expr(left, env)?, evaluate_expr(right, env)?) {
                (VariableValue::Num { value: left }, VariableValue::Num { value: right }) => {
                    Ok(VariableValue::Num {
                        value: left - right,
                    })
                }
                _ => Err(InterpreterError::operands_must_be_numbers_error(1)),
            }
        }
        Expr::Multiply { left, right } => {
            match (evaluate_expr(left, env)?, evaluate_expr(right, env)?) {
                (VariableValue::Num { value: left }, VariableValue::Num { value: right }) => {
                    Ok(VariableValue::Num {
                        value: left * right,
                    })
                }
                _ => Err(InterpreterError::operands_must_be_numbers_error(1)),
            }
        }
        Expr::Divide { left, right } => {
            match (evaluate_expr(left, env)?, evaluate_expr(right, env)?) {
                (VariableValue::Num { value: left }, VariableValue::Num { value: right }) => {
                    Ok(VariableValue::Num {
                        value: left / right,
                    })
                }
                _ => Err(InterpreterError::operands_must_be_numbers_error(1)),
            }
        }
    }
}
