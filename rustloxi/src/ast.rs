use crate::errors::{InterpreterError, Result};
use crate::{
    runtime::VariableValue,
    scanner::models::{Token, TokenInfo},
};

// TODO: factory methods can be pub(super) to restrict usage only for parent parser module
#[derive(Debug, Clone, PartialEq)]
pub struct ExprInfo {
    pub expr: Box<Expr>,
    pub line: u32,
    pub id: usize,
}

impl ExprInfo {
    pub fn new(id: usize, expr: Expr, line: u32) -> Self {
        ExprInfo {
            id,
            expr: Box::new(expr),
            line,
        }
    }

    pub fn binary(
        pos: usize,
        id: usize,
        left: ExprInfo,
        operator: &TokenInfo,
        right: ExprInfo,
    ) -> Result<ExprInfo> {
        match operator.token {
            Token::EqualEqual => Ok(ExprInfo::new(
                id,
                Expr::Equals { left, right },
                operator.line,
            )),
            Token::BangEqual => Ok(ExprInfo::new(
                id,
                Expr::NotEquals { left, right },
                operator.line,
            )),
            Token::Greater => Ok(ExprInfo::new(
                id,
                Expr::Greater { left, right },
                operator.line,
            )),
            Token::GreaterEqual => Ok(ExprInfo::new(
                id,
                Expr::GreaterEquals { left, right },
                operator.line,
            )),
            Token::Less => Ok(ExprInfo::new(id, Expr::Less { left, right }, operator.line)),
            Token::LessEqual => Ok(ExprInfo::new(
                id,
                Expr::LessEquals { left, right },
                operator.line,
            )),
            Token::Minus => Ok(ExprInfo::new(
                id,
                Expr::Minus { left, right },
                operator.line,
            )),
            Token::Plus => Ok(ExprInfo::new(id, Expr::Plus { left, right }, operator.line)),
            Token::Star => Ok(ExprInfo::new(
                id,
                Expr::Multiply { left, right },
                operator.line,
            )),
            Token::Slash => Ok(ExprInfo::new(
                id,
                Expr::Divide { left, right },
                operator.line,
            )),
            Token::Or => Ok(ExprInfo::new(id, Expr::Or { left, right }, operator.line)),
            Token::And => Ok(ExprInfo::new(id, Expr::And { left, right }, operator.line)),
            _ => Err(InterpreterError::parser_error(
                pos,
                operator,
                format!("'{}' is not a binary operator", operator.token.lexeme()),
            )),
        }
    }

    pub fn unary(pos: usize, id: usize, operator: &TokenInfo, arg: ExprInfo) -> Result<ExprInfo> {
        match operator.token {
            Token::Bang => Ok(ExprInfo::new(id, Expr::Not { expr: arg }, operator.line)),
            Token::Minus => Ok(ExprInfo::new(
                id,
                Expr::Negative { expr: arg },
                operator.line,
            )),
            _ => Err(InterpreterError::parser_error(
                pos,
                operator,
                format!("'{}' is not an unary operator", operator.token.lexeme()),
            )),
        }
    }

    pub fn assignment(id: usize, name: String, line: u32, value: ExprInfo) -> ExprInfo {
        ExprInfo::new(id, Expr::Assignment { name, value }, line)
    }

    pub fn call(line: u32, id: usize, callee: ExprInfo, args: Vec<ExprInfo>) -> ExprInfo {
        ExprInfo::new(id, Expr::Call { callee, args }, line)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Grouping {
        expr: ExprInfo,
    },

    Literal {
        value: VariableValue,
    },
    Nil,
    Variable {
        name: String,
    },
    Assignment {
        name: String,
        value: ExprInfo,
    },

    // Unary
    // -a
    Negative {
        expr: ExprInfo,
    },

    // !a
    Not {
        expr: ExprInfo,
    },

    //Binary
    Equals {
        left: ExprInfo,
        right: ExprInfo,
    },
    NotEquals {
        left: ExprInfo,
        right: ExprInfo,
    },
    Less {
        left: ExprInfo,
        right: ExprInfo,
    },
    LessEquals {
        left: ExprInfo,
        right: ExprInfo,
    },
    Greater {
        left: ExprInfo,
        right: ExprInfo,
    },
    GreaterEquals {
        left: ExprInfo,
        right: ExprInfo,
    },

    Plus {
        left: ExprInfo,
        right: ExprInfo,
    },
    Minus {
        left: ExprInfo,
        right: ExprInfo,
    },
    Multiply {
        left: ExprInfo,
        right: ExprInfo,
    },
    Divide {
        left: ExprInfo,
        right: ExprInfo,
    },

    Or {
        left: ExprInfo,
        right: ExprInfo,
    },
    And {
        left: ExprInfo,
        right: ExprInfo,
    },

    Call {
        callee: ExprInfo,
        args: Vec<ExprInfo>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expr {
        expr: ExprInfo,
    },
    Print {
        expr: ExprInfo,
    },
    Var {
        line: u32,
        name: String,
        initializer: Option<ExprInfo>,
    },
    Block {
        statements: Vec<Stmt>,
    },
    If {
        condition: ExprInfo,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: ExprInfo,
        stmt: Box<Stmt>,
    },

    Function {
        line: u32,
        name: String,
        params: Vec<(u32, String)>,
        body: Vec<Stmt>,
    },

    Return {
        line: u32,
        value: Option<ExprInfo>,
    },
}
