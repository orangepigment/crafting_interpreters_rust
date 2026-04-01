mod errors;
mod scanner;

use std::{
    fs,
    io::{self, Write},
    process::ExitCode,
};

fn run_file(filepath: &str) -> ExitCode {
    let program = fs::read_to_string(filepath).expect("BOOOM");
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
        io::stdout().flush().expect("BOOOM");

        let mut command = String::new();
        stdin
            .read_line(&mut command)
            .expect("Failed readline - HANDLE properly");

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

    for t in tokens {
        println!("{}", t.token);
    }

    Ok(())
}

fn main() -> ExitCode {
    let mut args = std::env::args();

    //There is at least 1 arg - programm name
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
