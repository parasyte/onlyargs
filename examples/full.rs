use error_iter::ErrorIter as _;
use onlyargs::{CliError, OnlyArgs};
use std::{ffi::OsString, path::PathBuf, process::ExitCode};

#[derive(Debug)]
struct Args {
    username: String,
    output: Option<PathBuf>,
    numbers: Vec<i32>,
    help: bool,
    version: bool,
}

impl OnlyArgs for Args {
    fn help() -> &'static str {
        concat!(
            env!("CARGO_PKG_NAME"),
            " v",
            env!("CARGO_PKG_VERSION"),
            "\n",
            env!("CARGO_PKG_DESCRIPTION"),
            "\n\n",
            "A basic argument parsing example with `onlyargs`.\n",
            "Sums a list of numbers and writes the result to a file or standard output.\n",
            "\nUsage:\n  ",
            env!("CARGO_BIN_NAME"),
            " [flags] [options] [numbers...]\n",
            "\nFlags:\n",
            "  -u --username <name>  Your username.\n",
            "  -o --output [path]    Output file path.\n",
            "\nOptions:\n",
            "  -h --help     Show this help message.\n",
            "  --version     Show the application version.\n",
            "\nNumbers:\n",
            "  A list of numbers to sum.\n",
        )
    }

    fn parse(args: Vec<OsString>) -> Result<Self, CliError> {
        let mut username = None;
        let mut output = None;
        let mut numbers = vec![];
        let mut help = false;
        let mut version = false;

        fn missing(s: OsString) -> CliError {
            CliError::MissingValue(s.into_string().unwrap())
        }

        let mut it = args.into_iter();
        while let Some(s) = it.next() {
            match s.to_str() {
                Some("--username") | Some("-u") => {
                    let name = it
                        .next()
                        .ok_or_else(|| missing(s))?
                        .into_string()
                        .map_err(|err| CliError::ParseStrError("username".to_string(), err))?;

                    username = Some(name);
                }
                Some("--output") | Some("-o") => {
                    output = Some(it.next().ok_or_else(|| missing(s))?.into());
                }
                Some("--help") | Some("-h") => {
                    help = true;
                }
                Some("--version") => {
                    version = true;
                }
                Some("--") => {
                    // Parse all positional arguments as i32.
                    let nums = it.map(|arg| {
                        arg.clone()
                            .into_string()
                            .map_err(|_| {
                                CliError::ParseStrError("<POSITIONAL>".to_string(), arg.clone())
                            })
                            .and_then(|num| {
                                num.parse::<i32>().map_err(|err| {
                                    CliError::ParseIntError("<POSITIONAL>".to_string(), arg, err)
                                })
                            })
                    });

                    if let Some(err) = nums.clone().find_map(|res| res.err()) {
                        return Err(err);
                    }
                    numbers.extend(
                        nums.into_iter()
                            .filter_map(|res| res.ok())
                            .collect::<Vec<_>>(),
                    );

                    break;
                }
                Some(num) => {
                    numbers.push(num.parse().map_err(|err| {
                        CliError::ParseIntError("<POSITIONAL>".to_string(), s, err)
                    })?);
                }
                None => return Err(onlyargs::CliError::Unknown(s)),
            }
        }

        // Required arguments.
        let username = username.ok_or_else(|| CliError::MissingRequired("username".to_string()))?;

        Ok(Self {
            username,
            output,
            numbers,
            help,
            version,
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

    // Handle `help` and `version` options.
    if args.help {
        args.show_help_and_exit();
    } else if args.version {
        args.show_version_and_exit();
    }

    println!("Hello, {}!", args.username);

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
    println!();
    dbg!(args);

    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            if matches!(err, Error::Cli(_)) {
                eprintln!("{}", Args::help());
            }

            eprintln!("Error: {err}");
            for source in err.sources().skip(1) {
                eprintln!("  Caused by: {source}");
            }

            ExitCode::FAILURE
        }
    }
}
