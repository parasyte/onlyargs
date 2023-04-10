use error_iter::ErrorIter as _;
use onlyargs::{traits::*, CliError, OnlyArgs};
use std::{ffi::OsString, path::PathBuf, process::ExitCode};

#[derive(Debug)]
struct Args {
    username: String,
    output: Option<PathBuf>,
    numbers: Vec<i32>,
    width: i32,
    verbose: bool,
}

impl OnlyArgs for Args {
    const HELP: &'static str = concat!(
        env!("CARGO_PKG_NAME"),
        " v",
        env!("CARGO_PKG_VERSION"),
        "\n",
        env!("CARGO_PKG_DESCRIPTION"),
        "\n\n",
        "A basic argument parsing example with `onlyargs`.\n",
        "Sums a list of numbers and writes the result to a file or standard output.\n",
        "\nUsage:\n  cargo run --example full -- [flags] [options] [numbers...]\n",
        "\nFlags:\n",
        "  -h --help     Show this help message.\n",
        "  -V --version  Show the application version.\n",
        "  -v --verbose  Enable verbose output.\n",
        "\nOptions:\n",
        "  -u --username STRING  Your username. [required]\n",
        "  -o --output PATH      Output file path.\n",
        "  -w --width NUMBER     Set the width. [default: 42]\n",
        "\nNumbers:\n",
        "  A list of numbers to sum.\n",
    );

    fn parse(args: Vec<OsString>) -> Result<Self, CliError> {
        let mut username = None;
        let mut output = None;
        let mut numbers = vec![];
        let mut width = 42;
        let mut verbose = false;

        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            match arg.to_str() {
                Some("--help") | Some("-h") => Self::help(),
                Some("--version") | Some("-V") => Self::version(),
                Some(name @ "--username") | Some(name @ "-u") => {
                    username = Some(args.next().parse_str(name)?);
                }
                Some(name @ "--output") | Some(name @ "-o") => {
                    output = Some(args.next().parse_path(name)?);
                }
                Some(name @ "--width") | Some(name @ "-w") => {
                    width = args.next().parse_int(name)?;
                }
                Some("--verbose") | Some("-v") => {
                    verbose = true;
                }
                Some("--") => {
                    // Parse all positional arguments as i32.
                    for arg in args {
                        numbers.push(arg.parse_int("<POSITIONAL>")?);
                    }
                    break;
                }
                Some(_) => {
                    numbers.push(arg.parse_int("<POSITIONAL>")?);
                }
                None => return Err(onlyargs::CliError::Unknown(arg)),
            }
        }

        Ok(Self {
            username: username.required("--username")?,
            output,
            numbers,
            width,
            verbose,
        })
    }
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
