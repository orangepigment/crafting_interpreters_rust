use std::fmt;

pub type Result<T> = std::result::Result<T, InterpreterError>;

// TODO: make constants for typical errors
// Define our error types. These may be customized for our error handling cases.
// Now we will be able to write our own errors, defer to an underlying error
// implementation, or do something in between.
#[derive(Debug)]
pub struct InterpreterError {
    line: u32,
    location: String,
    message: String,
}

impl InterpreterError {
    // TODO: add a function for building errors without location
    pub fn new(line: u32, location: String, message: String) -> InterpreterError {
        InterpreterError {
            line,
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
        write!(
            f,
            "[line {0}] Error {1}: {2}",
            self.line, self.message, self.location
        )
    }
}
