use error_iter::ErrorIter as _;
use onlyargs::{CliError, OnlyArgs};
use std::{ffi::OsString, process::ExitCode};

#[derive(Debug)]
struct Args {
    verbose: bool,
}

impl OnlyArgs for Args {
    const HELP: &'static str = onlyargs::impl_help!();
    const VERSION: &'static str = onlyargs::impl_version!();

    fn parse(args: Vec<OsString>) -> Result<Self, CliError> {
        let mut verbose = false;

        for arg in args.into_iter() {
            match arg.to_str() {
                Some("--help") | Some("-h") => Self::help(),
                Some("--version") | Some("-V") => Self::version(),
                Some("--verbose") | Some("-v") => {
                    verbose = true;
                }
                Some("--") => break,
                _ => return Err(CliError::Unknown(arg)),
            }
        }

        Ok(Self { verbose })
    }
}

fn run() -> Result<(), CliError> {
    let args: Args = onlyargs::parse()?;

    println!("Arguments parsed successfully!");

    if args.verbose {
        println!("Verbose output is enabled");
    }

    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{}", Args::HELP);

            eprintln!("Error: {err}");
            for source in err.sources().skip(1) {
                eprintln!("  Caused by: {source}");
            }

            ExitCode::FAILURE
        }
    }
}
