mod ast;
mod errors;
mod interpreter;
mod parser;
mod runtime;
mod scanner;

use std::{
    fs,
    io::{self, Write},
    process::ExitCode,
};

use crate::{interpreter::Interpreter, parser::Parser};

fn run_file(filepath: &str) -> ExitCode {
    let program = fs::read_to_string(filepath).expect("Failed to read the source file");
    run(&mut Interpreter::new(), program.as_str())
}

fn run_prompt() {
    let mut interpreter = Interpreter::new();
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
            run(&mut interpreter, command);
        }
    }
}

// FIX: to keep env in REPL interpreter must be converted into a param
// So declare it in the upper level
fn run(interpreter: &mut Interpreter, source: &str) -> ExitCode {
    let Ok(tokens) = scanner::scan_tokens(source).inspect_err(|e| eprintln!("{e}")) else {
        return ExitCode::from(65);
    };

    let mut parser = Parser::new();
    let Some(stmts) = parser.parse(&tokens) else {
        return ExitCode::from(65);
    };

    interpreter.interpret(&stmts).map_or_else(
        |e| {
            eprintln!("{e}");
            ExitCode::from(70)
        },
        |_| ExitCode::SUCCESS,
    )
}

fn main() -> ExitCode {
    let mut args = std::env::args();

    //There is at least 1 arg - program name
    match args.len() {
        3.. => {
            println!("Usage: rustloxi [script]");
            ExitCode::from(64)
        }
        2 => run_file(args.nth(1).unwrap().as_str()),
        _ => {
            run_prompt();
            ExitCode::SUCCESS
        }
    }
}
