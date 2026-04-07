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
}

/*pub fn render_ast(root: &Expr) -> String {
    let mut output = String::new();
    render_ast_loop(root, &mut output);
    output
}

fn render_ast_loop(current_expr: &Expr, output: &mut String) {
    match current_expr {
        Expr::Literal { value } => output.push_str(value.to_string().as_str()),
        Expr::Nil => output.push_str("nil"),
        Expr::Variable { name } => output.push_str(name),
        Expr::Assignment { name, value } => {
            output.push_str(&format!("(= {name}"));
            render_ast_loop(value, output);
        }
        Expr::Grouping { expr } => {
            output.push_str("(group ");
            render_ast_loop(expr, output);
            output.push(')');
        }
        Expr::Negative { expr } => {
            output.push_str("(- ");
            render_ast_loop(expr, output);
            output.push(')');
        }
        Expr::Not { expr } => {
            output.push_str("(! ");
            render_ast_loop(expr, output);
            output.push(')');
        }
        Expr::Equals { left, right } => {
            output.push_str("(== ");
            render_ast_loop(left, output);
            output.push(' ');
            render_ast_loop(right, output);
            output.push(')');
        }
        Expr::NotEquals { left, right } => {
            output.push_str("(!= ");
            render_ast_loop(left, output);
            output.push(' ');
            render_ast_loop(right, output);
            output.push(')');
        }
        Expr::Less { left, right } => {
            output.push_str("(< ");
            render_ast_loop(left, output);
            output.push(' ');
            render_ast_loop(right, output);
            output.push(')');
        }
        Expr::LessEquals { left, right } => {
            output.push_str("(<= ");
            render_ast_loop(left, output);
            output.push(' ');
            render_ast_loop(right, output);
            output.push(')');
        }
        Expr::Greater { left, right } => {
            output.push_str("(> ");
            render_ast_loop(left, output);
            output.push(' ');
            render_ast_loop(right, output);
            output.push(')');
        }
        Expr::GreaterEquals { left, right } => {
            output.push_str("(>= ");
            render_ast_loop(left, output);
            output.push(' ');
            render_ast_loop(right, output);
            output.push(')');
        }
        Expr::Plus { left, right } => {
            output.push_str("(+ ");
            render_ast_loop(left, output);
            output.push(' ');
            render_ast_loop(right, output);
            output.push(')');
        }
        Expr::Minus { left, right } => {
            output.push_str("(- ");
            render_ast_loop(left, output);
            output.push(' ');
            render_ast_loop(right, output);
            output.push(')');
        }
        Expr::Multiply { left, right } => {
            output.push_str("(* ");
            render_ast_loop(left, output);
            output.push(' ');
            render_ast_loop(right, output);
            output.push(')');
        }
        Expr::Divide { left, right } => {
            output.push_str("(/ ");
            render_ast_loop(left, output);
            output.push(' ');
            render_ast_loop(right, output);
            output.push(')');
        }
    }
}

pub fn render_reverse_polish_notation(root: &Expr) -> String {
    let mut output = String::new();
    render_reverse_polish_notation_loop(root, &mut output);
    output
}

fn render_reverse_polish_notation_loop(current_expr: &Expr, output: &mut String) {
    match current_expr {
        Expr::Literal { value } => output.push_str(value.to_string().as_str()),
        Expr::Nil => output.push_str("nil"),
        Expr::Variable { name } => output.push_str(name),
        Expr::Assignment { name, value } => {
            output.push_str(&format!(" {name} "));
            output.push_str(" = ");
            render_reverse_polish_notation_loop(value, output);
        }
        Expr::Grouping { expr } => {
            render_reverse_polish_notation_loop(expr, output);
        }
        Expr::Negative { expr } => {
            render_reverse_polish_notation_loop(expr, output);
            output.push_str(" - ");
        }
        Expr::Not { expr } => {
            render_reverse_polish_notation_loop(expr, output);
            output.push_str(" ! ");
        }
        Expr::Equals { left, right } => {
            render_reverse_polish_notation_loop(left, output);
            output.push(' ');
            render_reverse_polish_notation_loop(right, output);
            output.push_str(" == ");
        }
        Expr::NotEquals { left, right } => {
            render_reverse_polish_notation_loop(left, output);
            render_reverse_polish_notation_loop(right, output);
            output.push_str(" != ");
        }
        Expr::Less { left, right } => {
            render_reverse_polish_notation_loop(left, output);
            output.push(' ');
            render_reverse_polish_notation_loop(right, output);
            output.push_str(" < ");
        }
        Expr::LessEquals { left, right } => {
            render_reverse_polish_notation_loop(left, output);
            output.push(' ');
            render_reverse_polish_notation_loop(right, output);
            output.push_str(" <= ");
        }
        Expr::Greater { left, right } => {
            render_reverse_polish_notation_loop(left, output);
            output.push(' ');
            render_reverse_polish_notation_loop(right, output);
            output.push_str(" > ");
        }
        Expr::GreaterEquals { left, right } => {
            render_reverse_polish_notation_loop(left, output);
            output.push(' ');
            render_reverse_polish_notation_loop(right, output);
            output.push_str(" >=");
        }
        Expr::Plus { left, right } => {
            render_reverse_polish_notation_loop(left, output);
            output.push(' ');
            render_reverse_polish_notation_loop(right, output);
            output.push_str(" +");
        }
        Expr::Minus { left, right } => {
            render_reverse_polish_notation_loop(left, output);
            output.push(' ');
            render_reverse_polish_notation_loop(right, output);
            output.push_str(" -");
        }
        Expr::Multiply { left, right } => {
            render_reverse_polish_notation_loop(left, output);
            output.push(' ');
            render_reverse_polish_notation_loop(right, output);
            output.push_str(" *");
        }
        Expr::Divide { left, right } => {
            render_reverse_polish_notation_loop(left, output);
            output.push(' ');
            render_reverse_polish_notation_loop(right, output);
            output.push_str(" /");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ast_rendering() {
        let expr = Expr::Multiply {
            left: Box::new(Expr::Negative {
                expr: Box::new(Expr::Literal {
                    value: VariableValue::Num { value: 123_f64 },
                }),
            }),
            right: Box::new(Expr::Grouping {
                expr: Box::new(Expr::Literal {
                    value: VariableValue::Num { value: 45.67 },
                }),
            }),
        };

        assert_eq!(String::from("(* (- 123) (group 45.67))"), render_ast(&expr));
        assert_eq!(String::from("nil"), render_ast(&Expr::Nil));
    }

    #[test]
    fn reverse_polish_notation() {
        let expr = Expr::Multiply {
            left: Box::new(Expr::Plus {
                left: Box::new(Expr::Literal {
                    value: VariableValue::Num { value: 1_f64 },
                }),
                right: Box::new(Expr::Literal {
                    value: VariableValue::Num { value: 2_f64 },
                }),
            }),
            right: Box::new(Expr::Minus {
                left: Box::new(Expr::Literal {
                    value: VariableValue::Num { value: 4_f64 },
                }),
                right: Box::new(Expr::Literal {
                    value: VariableValue::Num { value: 3_f64 },
                }),
            }),
        };

        assert_eq!(
            String::from("1 2 + 4 3 - *"),
            render_reverse_polish_notation(&expr).trim_end()
        );
    }
}*/
