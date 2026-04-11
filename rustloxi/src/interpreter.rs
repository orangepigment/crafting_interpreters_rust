use std::collections::HashMap;

use crate::errors::{InterpreterError, Result};

use crate::runtime::{CLOCK, LoxCallable};
use crate::{
    ast::{Expr, ExprInfo, Stmt},
    runtime::VariableValue,
};

pub struct Interpreter {
    envs: Vec<HashMap<String, VariableValue>>,
    scope: usize,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let mut globals = HashMap::new();
        globals.insert(String::from("clock"), CLOCK);

        Interpreter {
            envs: vec![globals],
            scope: 0,
        }
    }

    fn get(&self, scope: usize, name: &str, line: u32) -> Result<VariableValue> {
        let mut scope = scope;
        loop {
            match self.envs[scope].get(name) {
                Some(value) => break Ok(value.clone()),
                None => {
                    if scope == 0 {
                        break Err(InterpreterError::runtime_error(
                            line,
                            format!("Undefined variable '{name}'."),
                        ));
                    } else {
                        scope -= 1;
                    }
                }
            }
        }
    }

    pub fn define(&mut self, scope: usize, name: String, value: VariableValue) {
        self.envs[scope].insert(name, value);
    }

    fn assign(
        &mut self,
        scope: usize,
        name: String,
        value: VariableValue,
        line: u32,
    ) -> Result<()> {
        let mut scope = scope;
        loop {
            // We don't use Entry API to avoid cloning name
            #[allow(clippy::map_entry)]
            if self.envs[scope].contains_key(&name) {
                self.envs[scope].insert(name, value);
                break Ok(());
            } else if scope == 0 {
                break Err(InterpreterError::runtime_error(
                    line,
                    format!("Undefined variable '{name}'."),
                ));
            } else {
                scope -= 1;
            }
        }
    }

    pub fn scope(&mut self) {
        self.envs.push(HashMap::new());
        self.scope += 1;
    }

    pub fn unscope(&mut self) {
        self.envs.pop();
        self.scope -= 1;
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

                self.define(self.scope, name.clone(), value);

                Ok(())
            }
            Stmt::Block { statements } => {
                self.scope();
                self.execute_block(statements)?;
                self.unscope();
                Ok(())
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
                self.define(
                    self.scope,
                    name.to_string(),
                    VariableValue::Function { raw: function },
                );

                Ok(())
            }
            Stmt::Return { line, value } => {
                // TODO: add Interpreter::ReturnErroe for unwinding. We don't need to return env from functions

                let value = match value {
                    Some(v) => self.evaluate_expr(v)?,
                    None => VariableValue::Nil,
                };

                Err(InterpreterError::return_value(value))
            }
        }
    }

    pub fn execute_block(&mut self, statements: &[Stmt]) -> Result<()> {
        for stmt in statements {
            self.execute_stmt(stmt)?;
        }

        Ok(())
    }

    fn evaluate_expr(&mut self, expr: &ExprInfo) -> Result<VariableValue> {
        match &*expr.expr {
            Expr::Grouping { expr } => self.evaluate_expr(expr),
            Expr::Literal { value } => Ok(value.clone()),
            Expr::Nil => Ok(VariableValue::Nil),
            Expr::Variable { name } => self.get(self.scope, name, expr.line),
            Expr::Assignment { name, value } => {
                let value = self.evaluate_expr(value)?;
                self.assign(self.scope, name.clone(), value.clone(), expr.line)?;

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

                let mut function = evaled_callee.into_function(callee.line)?;

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

                function.call(self, callee.line, &evaled_args)
            }
        }
    }
}
