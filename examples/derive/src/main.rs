//! This example is functionally identical to the `onlyargs` "full" example.
//!
//! It shows that if you can use `derive` macros, a lot of boilerplate can be scrubbed away!

use error_iter::ErrorIter as _;
use onlyargs::{CliError, OnlyArgs as _};
use onlyargs_derive::OnlyArgs;
use std::{path::PathBuf, process::ExitCode};

/// A basic argument parsing example with `onlyargs_derive`.
/// Sums a list of numbers and writes the result to a file or standard output.
#[derive(Clone, Debug, Eq, PartialEq, OnlyArgs)]
struct Args {
    /// Your username.
    username: String,

    /// Output file path.
    output: Option<PathBuf>,

    /// A list of numbers to sum.
    numbers: Vec<i32>,

    /// Set the width.
    #[default(42)]
    width: i32,

    /// Enable verbose output.
    verbose: bool,
}

#[derive(Debug)]
enum Error {
    Cli(CliError),
    Io(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cli(_) => write!(f, "Argument parsing error"),
            Self::Io(_) => write!(f, "I/O error"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Cli(err) => Some(err),
            Self::Io(err) => Some(err),
        }
    }
}

impl From<CliError> for Error {
    fn from(value: CliError) -> Self {
        Self::Cli(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

fn run() -> Result<(), Error> {
    let args: Args = onlyargs::parse()?;

    println!("Hello, {}!", args.username);
    println!("The width is {}.", args.width);

    // Do some work.
    let numbers = &args
        .numbers
        .iter()
        .map(|num| format!(" + {num}"))
        .collect::<String>();

    if let Some(numbers) = numbers.strip_prefix(" + ") {
        let sum: i32 = args.numbers.iter().sum();
        let msg = format!("The sum of {numbers} is {sum}");

        if let Some(path) = &args.output {
            std::fs::write(path, msg + "\n")?;
            println!("Sums written to {path:?}");
        } else {
            println!("{msg}");
        }
    }

    // And finally some debug info.
    if args.verbose {
        println!();
        dbg!(args);
    }

    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            if matches!(err, Error::Cli(_)) {
                eprintln!("{}", Args::HELP);
            }

            eprintln!("Error: {err}");
            for source in err.sources().skip(1) {
                eprintln!("  Caused by: {source}");
            }

            ExitCode::FAILURE
        }
    }
}
