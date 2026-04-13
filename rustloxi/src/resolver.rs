use std::collections::HashMap;

use crate::{
    ast::{Expr, ExprInfo, Stmt},
    errors::InterpreterError,
    interpreter::Interpreter,
};

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
    had_error: bool,
}

#[derive(Clone)]
enum FunctionType {
    None,
    Function,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Resolver {
            interpreter,
            scopes: vec![],
            current_function: FunctionType::None,
            had_error: false,
        }
    }

    fn resolve_block(&mut self, statements: &[Stmt]) {
        self.begin_scope();
        self.resolve(statements);
        self.end_scope();
    }

    pub fn resolve(&mut self, statements: &[Stmt]) {
        for stmt in statements {
            self.resolve_stmt(stmt);
        }
    }

    pub fn had_errors(&self) -> bool {
        self.had_error
    }

    // FIX: fix passing stub lines
    fn resolve_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr { expr } => self.resolve_expr(expr),
            Stmt::Print { expr } => self.resolve_expr(expr),
            Stmt::Var {
                name,
                initializer,
                line,
            } => {
                self.declare(name, *line);
                if let Some(init) = initializer {
                    self.resolve_expr(init);
                }
                self.define(name);
            }
            Stmt::Block { statements } => self.resolve_block(statements),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expr(condition);
                self.resolve_stmt(then_branch);

                if let Some(else_br) = else_branch {
                    self.resolve_stmt(else_br);
                }
            }
            Stmt::While { condition, stmt } => {
                self.resolve_expr(condition);
                self.resolve_stmt(stmt);
            }
            Stmt::Function {
                line,
                name,
                params,
                body,
            } => {
                self.declare(name, *line);
                self.define(name);

                self.resolve_function(params, body, FunctionType::Function);
            }
            Stmt::Return { line, value } => {
                if let FunctionType::None = self.current_function {
                    let error = InterpreterError::top_level_return(*line, String::from("'return'"));
                    eprintln!("{error}");
                    self.had_error = true;
                }

                if let Some(expr) = value {
                    self.resolve_expr(expr);
                }
            }
        }
    }

    fn resolve_expr(&mut self, expr: &ExprInfo) {
        match &*expr.expr {
            Expr::Grouping { expr } => self.resolve_expr(expr),
            Expr::Literal { value: _ } => {}
            Expr::Nil => {}
            Expr::Variable { name } => {
                match self.scopes.last() {
                    Some(scope) if scope.get(name).is_some_and(|b| !b) => {
                        let error =
                            InterpreterError::self_ref_initializer(expr.line, format!("'{name}'"));
                        eprintln!("{error}");
                        self.had_error = true;
                    }
                    _ => {}
                }

                self.resolve_local_expr(expr, name);
            }
            Expr::Assignment { name, value } => {
                self.resolve_expr(value);
                self.resolve_local_expr(expr, name);
            }
            Expr::Negative { expr } => self.resolve_expr(expr),
            Expr::Not { expr } => self.resolve_expr(expr),
            Expr::Equals { left, right } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            Expr::NotEquals { left, right } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            Expr::Less { left, right } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            Expr::LessEquals { left, right } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            Expr::Greater { left, right } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            Expr::GreaterEquals { left, right } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            Expr::Plus { left, right } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            Expr::Minus { left, right } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            Expr::Multiply { left, right } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            Expr::Divide { left, right } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            Expr::Or { left, right } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            Expr::And { left, right } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            Expr::Call { callee, args } => {
                self.resolve_expr(callee);
                for arg in args {
                    self.resolve_expr(arg);
                }
            }
        }
    }

    fn resolve_local_expr(&mut self, expr: &ExprInfo, name: &str) {
        for (scope_idx, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(name) {
                self.interpreter.resolve(expr, scope_idx);
                break;
            }
        }
    }

    fn resolve_function(&mut self, params: &[(u32, String)], body: &[Stmt], tpe: FunctionType) {
        let enclosing_function = self.current_function.clone();
        self.current_function = tpe;

        self.begin_scope();
        for (p_line, param) in params {
            self.declare(param, *p_line);
            self.define(param);
        }
        self.resolve(body);
        self.end_scope();
        self.current_function = enclosing_function;
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &str, line: u32) {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(name) {
                let error = InterpreterError::already_defined_variable(line, format!("'{name}'"));
                eprintln!("{error}");
                self.had_error = true;
            }
            scope.insert(name.to_string(), false);
        }
    }

    fn define(&mut self, name: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), true);
        }
    }
}
