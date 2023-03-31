use error_iter::ErrorIter as _;
use onlyargs::{CliError, OnlyArgs};
use std::{ffi::OsString, process::ExitCode};

struct Args {
    help: bool,
    version: bool,
}

impl OnlyArgs for Args {
    fn parse(args: Vec<OsString>) -> Result<Self, CliError> {
        let mut help = false;
        let mut version = false;

        for s in args.into_iter() {
            match s.to_str() {
                Some("--help") | Some("-h") => {
                    help = true;
                }
                Some("--version") | Some("-V") => {
                    version = true;
                }
                _ => return Err(CliError::Unknown(s)),
            }
        }

        Ok(Self { help, version })
    }
}

fn run() -> Result<(), CliError> {
    let args: Args = onlyargs::parse()?;

    // Handle `--help` and `--version` options.
    if args.help {
        args.help();
    } else if args.version {
        args.version();
    }

    println!("Arguments parsed successfully!");

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
