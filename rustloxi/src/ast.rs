use std::rc::Rc;

use crate::errors::{InterpreterError, Result};
use crate::scanner::models::{Token, TokenInfo};
use rustloxi::VariableValue;

// TODO: factory methods can be pub(super) to restrict usage only for parent parser module
pub enum Expr {
    Grouping { expr: Rc<Expr> },

    Literal { value: VariableValue },
    Nil,

    // Unary
    // -a
    Negative { expr: Rc<Expr> },

    // !a
    Not { expr: Rc<Expr> },

    //Binary
    Equals { left: Rc<Expr>, right: Rc<Expr> },
    NotEquals { left: Rc<Expr>, right: Rc<Expr> },
    Less { left: Rc<Expr>, right: Rc<Expr> },
    LessEquals { left: Rc<Expr>, right: Rc<Expr> },
    Greater { left: Rc<Expr>, right: Rc<Expr> },
    GreaterEquals { left: Rc<Expr>, right: Rc<Expr> },

    Plus { left: Rc<Expr>, right: Rc<Expr> },
    Minus { left: Rc<Expr>, right: Rc<Expr> },
    Multiply { left: Rc<Expr>, right: Rc<Expr> },
    Divide { left: Rc<Expr>, right: Rc<Expr> },
}

impl Expr {
    pub fn binary(left: Expr, operator: &TokenInfo, right: Expr) -> Result<Expr> {
        match operator.token {
            Token::EqualEqual => Ok(Expr::Equals {
                left: Rc::new(left),
                right: Rc::new(right),
            }),
            Token::BangEqual => Ok(Expr::NotEquals {
                left: Rc::new(left),
                right: Rc::new(right),
            }),
            Token::Greater => Ok(Expr::Greater {
                left: Rc::new(left),
                right: Rc::new(right),
            }),
            Token::GreaterEqual => Ok(Expr::GreaterEquals {
                left: Rc::new(left),
                right: Rc::new(right),
            }),
            Token::Less => Ok(Expr::Less {
                left: Rc::new(left),
                right: Rc::new(right),
            }),
            Token::LessEqual => Ok(Expr::LessEquals {
                left: Rc::new(left),
                right: Rc::new(right),
            }),
            Token::Minus => Ok(Expr::Minus {
                left: Rc::new(left),
                right: Rc::new(right),
            }),
            Token::Plus => Ok(Expr::Plus {
                left: Rc::new(left),
                right: Rc::new(right),
            }),
            Token::Star => Ok(Expr::Multiply {
                left: Rc::new(left),
                right: Rc::new(right),
            }),
            Token::Slash => Ok(Expr::Divide {
                left: Rc::new(left),
                right: Rc::new(right),
            }),
            _ => Err(InterpreterError::parser_error(
                operator,
                format!("'{}' is not a binary operator", operator.token.lexeme()),
            )),
        }
    }

    pub fn unary(operator: &TokenInfo, arg: Expr) -> Result<Expr> {
        match operator.token {
            Token::Bang => Ok(Expr::Not { expr: Rc::new(arg) }),
            Token::Minus => Ok(Expr::Negative { expr: Rc::new(arg) }),
            _ => Err(InterpreterError::parser_error(
                operator,
                format!("'{}' is not an unary operator", operator.token.lexeme()),
            )),
        }
    }
}

pub fn render_ast(root: &Expr) -> String {
    let mut output = String::new();
    render_ast_loop(root, &mut output);
    output
}

fn render_ast_loop(current_expr: &Expr, output: &mut String) {
    match current_expr {
        Expr::Literal { value } => output.push_str(value.to_string().as_str()),
        Expr::Nil => output.push_str("nil"),
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
            left: Rc::new(Expr::Negative {
                expr: Rc::new(Expr::Literal {
                    value: VariableValue::Num { value: 123_f64 },
                }),
            }),
            right: Rc::new(Expr::Grouping {
                expr: Rc::new(Expr::Literal {
                    value: VariableValue::Num { value: 45.67 },
                }),
            }),
        };

        assert_eq!(
            String::from("(* (- 123.0) (group 45.67))"),
            render_ast(&expr)
        );
        assert_eq!(String::from("nil"), render_ast(&Expr::Nil));
    }

    #[test]
    fn reverse_polish_notation() {
        let expr = Expr::Multiply {
            left: Rc::new(Expr::Plus {
                left: Rc::new(Expr::Literal {
                    value: VariableValue::Num { value: 1_f64 },
                }),
                right: Rc::new(Expr::Literal {
                    value: VariableValue::Num { value: 2_f64 },
                }),
            }),
            right: Rc::new(Expr::Minus {
                left: Rc::new(Expr::Literal {
                    value: VariableValue::Num { value: 4_f64 },
                }),
                right: Rc::new(Expr::Literal {
                    value: VariableValue::Num { value: 3_f64 },
                }),
            }),
        };

        assert_eq!(
            String::from("1.0 2.0 + 4.0 3.0 - *"),
            render_reverse_polish_notation(&expr).trim_end()
        );
    }
}
