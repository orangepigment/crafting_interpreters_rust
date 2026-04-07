use std::fmt;

use crate::scanner::models::{Token, TokenInfo};

pub type Result<T> = std::result::Result<T, InterpreterError>;

// Define our error types. These may be customized for our error handling cases.
// Now we will be able to write our own errors, defer to an underlying error
// implementation, or do something in between.

#[derive(Debug)]
pub enum InterpreterError {
    Scanner {
        line: u32,
        message: String,
    },

    Parser {
        line: u32,
        pos: usize,
        location: String,
        message: String,
    },

    Runtime {
        line: u32,
        message: String,
    },
}

impl InterpreterError {
    pub fn scanner_error(line: u32, message: String) -> InterpreterError {
        Self::Scanner { line, message }
    }

    pub fn parser_error(pos: usize, token: &TokenInfo, message: String) -> InterpreterError {
        let location = match token.token {
            Token::Eof => String::from("end"),
            _ => format!("'{}'", token.token.lexeme()),
        };

        Self::Parser {
            line: token.line,
            pos,
            location,
            message,
        }
    }

    pub fn runtime_error(line: u32, message: String) -> InterpreterError {
        InterpreterError::Runtime { line, message }
    }

    pub fn operands_must_be_numbers_error(line: u32) -> InterpreterError {
        InterpreterError::Runtime {
            line,
            message: String::from("Operands must be numbers."),
        }
    }
}

// Generation of an error is completely separate from how it is displayed.
// There's no need to be concerned about cluttering complex logic with the display style.
//
// Note that we don't store any extra info about the errors. This means we can't state
// which string failed to parse without modifying our types to carry that information.
impl fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            InterpreterError::Scanner { line, message } => {
                write!(f, "[line {line}] Error: {message}")
            }
            InterpreterError::Parser {
                line,
                pos: _,
                location,
                message,
            } => {
                write!(f, "[line {line}] Error at {location}: {message}")
            }
            InterpreterError::Runtime { line, message } => {
                write!(f, "{message}\n[line {line}]")
            }
        }
    }
}
