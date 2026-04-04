use std::fmt;

use crate::scanner::models::{Token, TokenInfo};

pub type Result<T> = std::result::Result<T, InterpreterError>;

// TODO: make constants for typical errors
// Define our error types. These may be customized for our error handling cases.
// Now we will be able to write our own errors, defer to an underlying error
// implementation, or do something in between.

#[derive(Debug)]
pub enum InterpreterError {
    ScannerError {
        line: u32,
        message: String,
    },

    ParserError {
        line: u32,
        location: String,
        message: String,
    },
}

impl InterpreterError {
    pub fn scanner_error(line: u32, message: String) -> InterpreterError {
        Self::ScannerError { line, message }
    }

    pub fn parser_error(token: &TokenInfo, message: String) -> InterpreterError {
        let location = match token.token {
            Token::Eof => String::from("end"),
            _ => format!("'{}'", token.token.lexeme()),
        };

        Self::ParserError {
            line: token.line,
            location,
            message,
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
            InterpreterError::ScannerError { line, message } => {
                write!(f, "[line {line}] Error: {message}")
            }
            InterpreterError::ParserError {
                line,
                location,
                message,
            } => {
                write!(f, "[line {line}] Error at {location}: {message}")
            }
        }
    }
}
