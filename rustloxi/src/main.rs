mod ast;
mod errors;
mod parser;
mod scanner;

use std::{
    fs,
    io::{self, Write},
    process::ExitCode,
};

use crate::ast::render_ast;

fn run_file(filepath: &str) -> ExitCode {
    let program = fs::read_to_string(filepath).expect("Failed to read the source file");
    run(program.as_str())
        .map(|_| ExitCode::SUCCESS)
        .inspect_err(|e| eprintln!("{e}"))
        .unwrap_or_else(|_| ExitCode::from(65))
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
    let expr = parser::parse(&tokens)?;

    println!("{}", render_ast(&expr));

    Ok(())
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
            run_file(args.nth(1).unwrap().as_str());
            ExitCode::SUCCESS
        }
        _ => {
            run_prompt();
            ExitCode::SUCCESS
        }
    }
}
