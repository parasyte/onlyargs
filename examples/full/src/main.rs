use error_iter::ErrorIter as _;
use onlyargs::{traits::*, CliError, OnlyArgs};
use onlyerror::Error;
use std::{ffi::OsString, fmt::Write as _, path::PathBuf, process::ExitCode};

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
        "\nUsage:\n  cargo run -p example-full -- [flags] [options] [numbers...]\n",
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
        "\nPlease consider becoming a sponsor 💖:\n",
        "  * https://github.com/sponsors/parasyte\n",
        "  * https://ko-fi.com/blipjoy\n",
        "  * https://patreon.com/blipjoy\n",
    );

    const VERSION: &'static str = onlyargs::impl_version!();

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

#[derive(Debug, Error)]
enum Error {
    /// Argument parsing error.
    Cli(#[from] CliError),

    /// I/O error.
    Io(#[from] std::io::Error),
}

fn run() -> Result<(), Error> {
    let args: Args = onlyargs::parse()?;

    println!("Hello, {}!", args.username);
    println!("The width is {}.", args.width);

    // Do some work.
    let numbers = &args.numbers.iter().fold(String::new(), |mut numbers, num| {
        write!(numbers, " + {num}").unwrap();
        numbers
    });

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
