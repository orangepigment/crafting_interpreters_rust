use crate::errors::{InterpreterError, Result};

use crate::{
    ast::{Expr, ExprInfo, Stmt},
    runtime::{Environment, LoxCallable, VariableValue},
};

pub struct Interpreter {
    env: Environment,
    scope: usize,
}

impl Interpreter {
    pub fn new() -> Self {
        let env = Environment::new();

        Interpreter { env, scope: 0 }
    }

    pub fn interpret(&mut self, stmts: &[Stmt]) -> Result<()> {
        for s in stmts {
            self.execute_stmt(s)?;
        }

        // We don't restore state and it allow us to save vars in REPL
        Ok(())
    }

    fn execute_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Expr { expr } => {
                self.evaluate_expr(expr)?;
                Ok(())
            }
            Stmt::Print { expr } => {
                let value = self.evaluate_expr(expr)?;
                println!("{value}");

                Ok(())
            }
            Stmt::Var { name, initializer } => {
                let value = match initializer {
                    Some(i) => self.evaluate_expr(i)?,
                    None => VariableValue::Nil,
                };

                self.env.define(self.scope, name.clone(), value);

                Ok(())
            }
            Stmt::Block { statements } => {
                self.scope = self.env.scope(self.scope);
                self.execute_block(statements)
                    .inspect(|_| {
                        self.scope = self.env.unscope(self.scope);
                    })
                    .inspect_err(|_| {
                        self.scope = self.env.unscope(self.scope);
                    })
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_res = self.evaluate_expr(condition)?;

                if cond_res.is_truthy() {
                    self.execute_stmt(then_branch)
                } else {
                    match else_branch {
                        Some(else_br) => self.execute_stmt(else_br),
                        None => Ok(()),
                    }
                }
            }
            Stmt::While {
                condition: expr,
                stmt,
            } => {
                while self.evaluate_expr(expr)?.is_truthy() {
                    self.execute_stmt(stmt)?;
                }

                Ok(())
            }
            Stmt::Function { name, params, body } => {
                let function = LoxCallable::Function {
                    name: name.to_string(),
                    params: params.to_vec(),
                    body: body.to_vec(),
                    closure: self.scope,
                };
                self.env.define(
                    self.scope,
                    name.to_string(),
                    VariableValue::Function { raw: function },
                );

                Ok(())
            }
            Stmt::Return { line: _, value } => {
                let value = match value {
                    Some(v) => self.evaluate_expr(v)?,
                    None => VariableValue::Nil,
                };

                Err(InterpreterError::return_value(value))
            }
        }
    }

    fn execute_block(&mut self, statements: &[Stmt]) -> Result<()> {
        for stmt in statements {
            self.execute_stmt(stmt)?
        }

        Ok(())
    }

    fn evaluate_expr(&mut self, expr: &ExprInfo) -> Result<VariableValue> {
        match &*expr.expr {
            Expr::Grouping { expr } => self.evaluate_expr(expr),
            Expr::Literal { value } => Ok(value.clone()),
            Expr::Nil => Ok(VariableValue::Nil),
            Expr::Variable { name } => self.env.get(self.scope, name, expr.line).cloned(),
            Expr::Assignment { name, value } => {
                let value = self.evaluate_expr(value)?;
                self.env
                    .assign(self.scope, name.clone(), value.clone(), expr.line)?;

                Ok(value)
            }
            Expr::Negative { expr } => match self.evaluate_expr(expr)? {
                VariableValue::Num { value } => Ok(VariableValue::Num { value: -value }),
                _ => Err(InterpreterError::runtime_error(
                    expr.line,
                    String::from("Operand must be a number."),
                )),
            },
            Expr::Not { expr } => {
                let result = self.evaluate_expr(expr)?;
                Ok(VariableValue::Boolean {
                    value: !result.is_truthy(),
                })
            }
            Expr::Equals { left, right } => Ok(VariableValue::Boolean {
                value: self.evaluate_expr(left)? == self.evaluate_expr(right)?,
            }),
            Expr::NotEquals { left, right } => Ok(VariableValue::Boolean {
                value: self.evaluate_expr(left)? != self.evaluate_expr(right)?,
            }),
            Expr::Less { left, right } => {
                match (self.evaluate_expr(left)?, self.evaluate_expr(right)?) {
                    (VariableValue::Num { value: left }, VariableValue::Num { value: right }) => {
                        Ok(VariableValue::Boolean {
                            value: left < right,
                        })
                    }
                    _ => Err(InterpreterError::operands_must_be_numbers_error(1)),
                }
            }
            Expr::LessEquals { left, right } => {
                match (self.evaluate_expr(left)?, self.evaluate_expr(right)?) {
                    (VariableValue::Num { value: left }, VariableValue::Num { value: right }) => {
                        Ok(VariableValue::Boolean {
                            value: left <= right,
                        })
                    }
                    _ => Err(InterpreterError::operands_must_be_numbers_error(1)),
                }
            }
            Expr::Greater { left, right } => {
                match (self.evaluate_expr(left)?, self.evaluate_expr(right)?) {
                    (VariableValue::Num { value: left }, VariableValue::Num { value: right }) => {
                        Ok(VariableValue::Boolean {
                            value: left > right,
                        })
                    }
                    _ => Err(InterpreterError::operands_must_be_numbers_error(1)),
                }
            }
            Expr::GreaterEquals { left, right } => {
                match (self.evaluate_expr(left)?, self.evaluate_expr(right)?) {
                    (VariableValue::Num { value: left }, VariableValue::Num { value: right }) => {
                        Ok(VariableValue::Boolean {
                            value: left >= right,
                        })
                    }
                    _ => Err(InterpreterError::operands_must_be_numbers_error(1)),
                }
            }
            Expr::Plus { left, right } => {
                match (self.evaluate_expr(left)?, self.evaluate_expr(right)?) {
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
                }
            }
            Expr::Minus { left, right } => {
                match (self.evaluate_expr(left)?, self.evaluate_expr(right)?) {
                    (VariableValue::Num { value: left }, VariableValue::Num { value: right }) => {
                        Ok(VariableValue::Num {
                            value: left - right,
                        })
                    }
                    _ => Err(InterpreterError::operands_must_be_numbers_error(1)),
                }
            }
            Expr::Multiply { left, right } => {
                match (self.evaluate_expr(left)?, self.evaluate_expr(right)?) {
                    (VariableValue::Num { value: left }, VariableValue::Num { value: right }) => {
                        Ok(VariableValue::Num {
                            value: left * right,
                        })
                    }
                    _ => Err(InterpreterError::operands_must_be_numbers_error(1)),
                }
            }
            Expr::Divide { left, right } => {
                match (self.evaluate_expr(left)?, self.evaluate_expr(right)?) {
                    (VariableValue::Num { value: left }, VariableValue::Num { value: right }) => {
                        Ok(VariableValue::Num {
                            value: left / right,
                        })
                    }
                    _ => Err(InterpreterError::operands_must_be_numbers_error(1)),
                }
            }
            Expr::Or { left, right } => {
                let left = self.evaluate_expr(left)?;

                if left.is_truthy() {
                    Ok(left)
                } else {
                    self.evaluate_expr(right)
                }
            }
            Expr::And { left, right } => {
                let left = self.evaluate_expr(left)?;

                if !left.is_truthy() {
                    Ok(left)
                } else {
                    self.evaluate_expr(right)
                }
            }
            Expr::Call { callee, args } => {
                let evaled_callee = self.evaluate_expr(callee)?;

                let mut evaled_args = vec![];
                for a in args {
                    evaled_args.push(self.evaluate_expr(a)?);
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

                self.execute_call(&function, callee.line, &evaled_args)
            }
        }
    }

    fn execute_call(
        &mut self,
        callable: &LoxCallable,
        line: u32,
        args: &[VariableValue],
    ) -> Result<VariableValue> {
        match callable {
            LoxCallable::Function {
                name: _,
                params,
                body,
                closure,
            } => {
                let call_place_scope = self.scope;
                self.scope = self.env.scope(*closure);

                for (p, a) in std::iter::zip(params, args) {
                    self.env.define(self.scope, p.to_string(), a.clone());
                }

                match self.execute_block(body).inspect_err(|_| {
                    // Free call scope and return to call place scope
                    self.env.unscope(self.scope);
                    self.scope = call_place_scope;
                }) {
                    Ok(_) => {
                        // Free call scope and return to call place scope
                        self.env.unscope(self.scope);
                        self.scope = call_place_scope;
                        Ok(VariableValue::Nil)
                    }
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
}
