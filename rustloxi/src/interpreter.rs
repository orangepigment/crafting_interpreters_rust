use crate::errors::{InterpreterError, Result};

use crate::runtime::{CLOCK, LoxCallable};
use crate::{
    ast::{Expr, ExprInfo, Stmt},
    runtime::{Environment, VariableValue},
};

pub fn interpret(stmts: &Vec<Stmt>) -> Result<()> {
    let mut global_env = Environment::new();
    global_env.define(String::from("clock"), CLOCK);

    for s in stmts {
        global_env = execute_stmt(s, global_env)?;
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
        Stmt::Block { statements } => execute_block(statements, env.scope()),
        Stmt::If {
            condition,
            then_branch,
            else_branch,
        } => {
            let cond_res = evaluate_expr(condition, &mut env)?;

            if cond_res.is_truthy() {
                execute_stmt(then_branch, env)
            } else {
                match else_branch {
                    Some(else_br) => execute_stmt(else_br, env),
                    None => Ok(env),
                }
            }
        }
        Stmt::While {
            condition: expr,
            stmt,
        } => {
            let mut env = env;
            while evaluate_expr(expr, &mut env)?.is_truthy() {
                env = execute_stmt(stmt, env)?;
            }

            Ok(env)
        }
        Stmt::Function { name, params, body } => {
            let function = LoxCallable::Function {
                name: name.to_string(),
                params: params.to_vec(),
                body: body.to_vec(),
            };
            env.define(name.to_string(), VariableValue::Function { raw: function });

            Ok(env)
        }
        Stmt::Return { line, value } => {
            // TODO: add Interpreter::ReturnErroe for unwinding. We don't need to return env from functions

            let value = match value {
                Some(v) => evaluate_expr(v, &mut env)?,
                None => VariableValue::Nil,
            };

            Err(InterpreterError::return_value(value))
        }
    }
}

pub fn execute_block(statements: &[Stmt], env: Environment) -> Result<Environment> {
    let mut env = env;

    for stmt in statements {
        env = execute_stmt(stmt, env)?;
    }

    Ok(env.unscope())
}

fn evaluate_expr(expr: &ExprInfo, env: &mut Environment) -> Result<VariableValue> {
    match &*expr.expr {
        Expr::Grouping { expr } => evaluate_expr(expr, env),
        Expr::Literal { value } => Ok(value.clone()),
        Expr::Nil => Ok(VariableValue::Nil),
        Expr::Variable { name } => env.get(name, expr.line).cloned(),
        Expr::Assignment { name, value } => {
            let value = evaluate_expr(value, env)?;
            env.assign(name.clone(), value.clone(), expr.line)?;

            Ok(value)
        }
        Expr::Negative { expr } => match evaluate_expr(expr, env)? {
            VariableValue::Num { value } => Ok(VariableValue::Num { value: -value }),
            _ => Err(InterpreterError::runtime_error(
                expr.line,
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
                expr.line,
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
        Expr::Or { left, right } => {
            let left = evaluate_expr(left, env)?;

            if left.is_truthy() {
                Ok(left)
            } else {
                evaluate_expr(right, env)
            }
        }
        Expr::And { left, right } => {
            let left = evaluate_expr(left, env)?;

            if !left.is_truthy() {
                Ok(left)
            } else {
                evaluate_expr(right, env)
            }
        }
        Expr::Call { callee, args } => {
            let evaled_callee = evaluate_expr(callee, env)?;

            let mut evaled_args = vec![];
            for a in args {
                evaled_args.push(evaluate_expr(a, env)?);
            }

            let function = evaled_callee.into_function(callee.line)?;

            if evaled_args.len() != function.arity() {
                return Err(InterpreterError::runtime_error(
                    callee.line,
                    format!(
                        "Expected {0} arguments but got {1}.",
                        function.arity(),
                        evaled_args.len()
                    ),
                ));
            }

            function.call(callee.line, &evaled_args)
        }
    }
}
