use crate::errors::{InterpreterError, Result};
use crate::{
    scanner::models::{Token, TokenInfo},
    state::VariableValue,
};

// TODO: factory methods can be pub(super) to restrict usage only for parent parser module
pub struct ExprInfo {
    pub expr: Box<Expr>,
    pub line: u32,
}

impl ExprInfo {
    pub fn new(expr: Expr, line: u32) -> ExprInfo {
        ExprInfo {
            expr: Box::new(expr),
            line,
        }
    }

    pub fn binary(left: ExprInfo, operator: &TokenInfo, right: ExprInfo) -> Result<ExprInfo> {
        match operator.token {
            Token::EqualEqual => Ok(ExprInfo::new(Expr::Equals { left, right }, operator.line)),
            Token::BangEqual => Ok(ExprInfo::new(
                Expr::NotEquals { left, right },
                operator.line,
            )),
            Token::Greater => Ok(ExprInfo::new(Expr::Greater { left, right }, operator.line)),
            Token::GreaterEqual => Ok(ExprInfo::new(
                Expr::GreaterEquals { left, right },
                operator.line,
            )),
            Token::Less => Ok(ExprInfo::new(Expr::Less { left, right }, operator.line)),
            Token::LessEqual => Ok(ExprInfo::new(
                Expr::LessEquals { left, right },
                operator.line,
            )),
            Token::Minus => Ok(ExprInfo::new(Expr::Minus { left, right }, operator.line)),
            Token::Plus => Ok(ExprInfo::new(Expr::Plus { left, right }, operator.line)),
            Token::Star => Ok(ExprInfo::new(Expr::Multiply { left, right }, operator.line)),
            Token::Slash => Ok(ExprInfo::new(Expr::Divide { left, right }, operator.line)),
            Token::Or => Ok(ExprInfo::new(Expr::Or { left, right }, operator.line)),
            Token::And => Ok(ExprInfo::new(Expr::And { left, right }, operator.line)),
            _ => Err(InterpreterError::parser_error(
                operator,
                format!("'{}' is not a binary operator", operator.token.lexeme()),
            )),
        }
    }

    pub fn unary(operator: &TokenInfo, arg: ExprInfo) -> Result<ExprInfo> {
        match operator.token {
            Token::Bang => Ok(ExprInfo::new(Expr::Not { expr: arg }, operator.line)),
            Token::Minus => Ok(ExprInfo::new(Expr::Negative { expr: arg }, operator.line)),
            _ => Err(InterpreterError::parser_error(
                operator,
                format!("'{}' is not an unary operator", operator.token.lexeme()),
            )),
        }
    }

    pub fn assignment(name: String, line: u32, value: ExprInfo) -> ExprInfo {
        ExprInfo::new(Expr::Assignment { name, value }, line)
    }
}

pub enum Expr {
    Grouping { expr: ExprInfo },

    Literal { value: VariableValue },
    Nil,
    Variable { name: String },
    Assignment { name: String, value: ExprInfo },

    // Unary
    // -a
    Negative { expr: ExprInfo },

    // !a
    Not { expr: ExprInfo },

    //Binary
    Equals { left: ExprInfo, right: ExprInfo },
    NotEquals { left: ExprInfo, right: ExprInfo },
    Less { left: ExprInfo, right: ExprInfo },
    LessEquals { left: ExprInfo, right: ExprInfo },
    Greater { left: ExprInfo, right: ExprInfo },
    GreaterEquals { left: ExprInfo, right: ExprInfo },

    Plus { left: ExprInfo, right: ExprInfo },
    Minus { left: ExprInfo, right: ExprInfo },
    Multiply { left: ExprInfo, right: ExprInfo },
    Divide { left: ExprInfo, right: ExprInfo },

    Or { left: ExprInfo, right: ExprInfo },
    And { left: ExprInfo, right: ExprInfo },
}

pub enum Stmt {
    Expr {
        expr: ExprInfo,
    },
    Print {
        expr: ExprInfo,
    },
    Var {
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
}
