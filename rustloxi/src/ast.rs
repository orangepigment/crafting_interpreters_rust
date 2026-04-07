use crate::errors::{InterpreterError, Result};
use crate::{
    scanner::models::{Token, TokenInfo},
    state::VariableValue,
};

// TODO: factory methods can be pub(super) to restrict usage only for parent parser module
// TODO: keep line number info inside expressions
pub enum Expr {
    Grouping { expr: Box<Expr> },

    Literal { value: VariableValue },
    Nil,
    Variable { name: String },
    Assignment { name: String, value: Box<Expr> },

    // Unary
    // -a
    Negative { expr: Box<Expr> },

    // !a
    Not { expr: Box<Expr> },

    //Binary
    Equals { left: Box<Expr>, right: Box<Expr> },
    NotEquals { left: Box<Expr>, right: Box<Expr> },
    Less { left: Box<Expr>, right: Box<Expr> },
    LessEquals { left: Box<Expr>, right: Box<Expr> },
    Greater { left: Box<Expr>, right: Box<Expr> },
    GreaterEquals { left: Box<Expr>, right: Box<Expr> },

    Plus { left: Box<Expr>, right: Box<Expr> },
    Minus { left: Box<Expr>, right: Box<Expr> },
    Multiply { left: Box<Expr>, right: Box<Expr> },
    Divide { left: Box<Expr>, right: Box<Expr> },
}

impl Expr {
    pub fn binary(left: Expr, operator: &TokenInfo, right: Expr) -> Result<Expr> {
        match operator.token {
            Token::EqualEqual => Ok(Expr::Equals {
                left: Box::new(left),
                right: Box::new(right),
            }),
            Token::BangEqual => Ok(Expr::NotEquals {
                left: Box::new(left),
                right: Box::new(right),
            }),
            Token::Greater => Ok(Expr::Greater {
                left: Box::new(left),
                right: Box::new(right),
            }),
            Token::GreaterEqual => Ok(Expr::GreaterEquals {
                left: Box::new(left),
                right: Box::new(right),
            }),
            Token::Less => Ok(Expr::Less {
                left: Box::new(left),
                right: Box::new(right),
            }),
            Token::LessEqual => Ok(Expr::LessEquals {
                left: Box::new(left),
                right: Box::new(right),
            }),
            Token::Minus => Ok(Expr::Minus {
                left: Box::new(left),
                right: Box::new(right),
            }),
            Token::Plus => Ok(Expr::Plus {
                left: Box::new(left),
                right: Box::new(right),
            }),
            Token::Star => Ok(Expr::Multiply {
                left: Box::new(left),
                right: Box::new(right),
            }),
            Token::Slash => Ok(Expr::Divide {
                left: Box::new(left),
                right: Box::new(right),
            }),
            _ => Err(InterpreterError::parser_error(
                operator,
                format!("'{}' is not a binary operator", operator.token.lexeme()),
            )),
        }
    }

    pub fn unary(operator: &TokenInfo, arg: Expr) -> Result<Expr> {
        match operator.token {
            Token::Bang => Ok(Expr::Not {
                expr: Box::new(arg),
            }),
            Token::Minus => Ok(Expr::Negative {
                expr: Box::new(arg),
            }),
            _ => Err(InterpreterError::parser_error(
                operator,
                format!("'{}' is not an unary operator", operator.token.lexeme()),
            )),
        }
    }

    pub fn assignment(name: String, value: Expr) -> Expr {
        Expr::Assignment {
            name,
            value: Box::new(value),
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

pub enum Stmt {
    Expr {
        expr: Expr,
    },
    Print {
        expr: Expr,
    },
    Var {
        name: String,
        initializer: Option<Expr>,
    },
    Block {
        statements: Vec<Stmt>,
    },
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
}
