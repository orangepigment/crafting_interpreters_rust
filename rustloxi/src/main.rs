mod ast;
mod errors;
mod interpreter;
mod parser;
mod scanner;
mod state;

use std::{
    fs,
    io::{self, Write},
    process::ExitCode,
};

use crate::interpreter::interpret;

fn run_file(filepath: &str) -> ExitCode {
    let program = fs::read_to_string(filepath).expect("Failed to read the source file");
    run(program.as_str())
        .map(|_| ExitCode::SUCCESS)
        .inspect_err(|e| eprintln!("{e}"))
        .unwrap_or_else(|e| {
            match e {
                errors::InterpreterError::Scanner { .. } => ExitCode::from(65),
                errors::InterpreterError::Parser { .. } => ExitCode::from(65),
                errors::InterpreterError::Runtime { .. } => ExitCode::from(70),
            }
        })
}

fn run_prompt() {
    let stdin = io::stdin();
    loop {
        print!("> ");
        // Explicitly flush the standard output
        io::stdout().flush().unwrap();

        let mut command = String::new();
        stdin
            .read_line(&mut command)
            .expect("Failed to read user input...");

        let command = command.trim_end();

        if command.is_empty() {
            println!("Bye-bye!");
            break;
        } else {
            if let Err(e) = run(command) {
                eprintln!("{e}");
            }
        }
    }
}

fn run(source: &str) -> errors::Result<()> {
    let tokens = scanner::scan_tokens(source)?;
    let stmts = parser::parse(&tokens)?;

    interpret(&stmts).inspect_err(|e| eprintln!("{e}"))
}

fn main() -> ExitCode {
    let mut args = std::env::args();

    //There is at least 1 arg - program name
    match args.len() {
        3.. => {
            println!("Usage: rustloxi [script]");
            ExitCode::from(64)
        }
        2 => {
            run_file(args.nth(1).unwrap().as_str())
        }
        _ => {
            run_prompt();
            ExitCode::SUCCESS
        }
    }
}
