//! Only argument parsing! Nothing more.
//!
//! `onlyargs` is an obsessively tiny argument parsing library. It provides a basic trait and a
//! helper function for parsing arguments from the environment.
//!
//! Implement the [`OnlyArgs`] trait on your own argument type or use the
//! [`onlyargs_derive`](https://docs.rs/onlyargs_derive) crate to generate an opinionated parser.

use std::{env, ffi::OsString, fmt::Display};

/// Argument parsing errors.
#[derive(Debug)]
pub enum CliError {
    /// An argument requires a value, but one was not provided.
    MissingValue(String),

    /// A required argument was not provided.
    MissingRequired(String),

    /// An argument requires a value, but parsing it as a `bool` failed.
    ParseBoolError(String, OsString, std::str::ParseBoolError),

    /// An argument requires a value, but parsing it as a `char` failed.
    ParseCharError(String, OsString, std::char::ParseCharError),

    /// An argument requires a value, but parsing it as a floating-point number failed.
    ParseFloatError(String, OsString, std::num::ParseFloatError),

    /// An argument requires a value, but parsing it as an integer failed.
    ParseIntError(String, OsString, std::num::ParseIntError),

    /// An argument requires a value, but parsing it as a `String` failed.
    ParseStrError(String, OsString),

    /// An unknown argument was provided.
    Unknown(OsString),
}

impl Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingValue(arg) => write!(f, "Missing value for argument `{arg}`"),
            Self::MissingRequired(arg) => write!(f, "Missing required argument `{arg}`"),
            Self::ParseBoolError(arg, value, _) => write!(
                f,
                "Bool parsing error for argument `{arg}`: value={value:?}"
            ),
            Self::ParseCharError(arg, value, _) => write!(
                f,
                "Char parsing error for argument `{arg}`: value={value:?}"
            ),
            Self::ParseFloatError(arg, value, _) => write!(
                f,
                "Float parsing error for argument `{arg}`: value={value:?}"
            ),
            Self::ParseIntError(arg, value, _) => {
                write!(f, "Int parsing error for argument `{arg}`: value={value:?}")
            }
            Self::ParseStrError(arg, value) => write!(
                f,
                "String parsing error for argument `{arg}`: value={value:?}"
            ),
            Self::Unknown(arg) => write!(f, "Unknown argument: {arg:?}"),
        }
    }
}

impl std::error::Error for CliError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ParseBoolError(_, _, err) => Some(err),
            Self::ParseCharError(_, _, err) => Some(err),
            Self::ParseFloatError(_, _, err) => Some(err),
            Self::ParseIntError(_, _, err) => Some(err),
            _ => None,
        }
    }
}

/// The primary argument parser trait.
///
/// This trait can be derived with the [`onlyargs_derive`](https://docs.rs/onlyargs_derive) crate.
///
/// See [`onlyargs::parse`] for more information.
pub trait OnlyArgs {
    /// Construct a type that implements this trait.
    ///
    /// Each argument is provided as an [`OsString`].
    fn parse(args: Vec<OsString>) -> Result<Self, CliError>
    where
        Self: Sized;

    /// Associated method that returns the application help string.
    fn help() -> &'static str {
        concat!(
            env!("CARGO_PKG_NAME"),
            " v",
            env!("CARGO_PKG_VERSION"),
            "\n",
            env!("CARGO_PKG_DESCRIPTION"),
            "\n",
        )
    }

    /// Print the application help string and exit the process.
    fn show_help_and_exit(&self) -> ! {
        eprintln!("{}", Self::help());
        std::process::exit(0);
    }

    /// Associated method that returns the application name and version.
    fn version() -> &'static str {
        concat!(
            env!("CARGO_PKG_NAME"),
            " v",
            env!("CARGO_PKG_VERSION"),
            "\n",
        )
    }

    /// Print the application name and version and exit the process.
    fn show_version_and_exit(&self) -> ! {
        eprintln!("{}", Self::version());
        std::process::exit(0);
    }
}

/// Type constructor for argument parser.
///
/// Given a type that implements [`OnlyArgs`], this function will construct the type from the
/// current environment.
///
/// # Example
///
/// ```
/// # use std::ffi::OsString;
/// # use onlyargs::{CliError, OnlyArgs};
/// struct Args {
///     help: bool,
///     version: bool,
/// }
///
/// impl OnlyArgs for Args {
///     fn parse(args: Vec<OsString>) -> Result<Self, CliError> {
///         let mut help = false;
///         let mut version = false;
///
///         for s in args.into_iter() {
///             match s.to_str() {
///                 Some("--help") | Some("-h") => {
///                     help = true;
///                 }
///                 Some("--version") => {
///                     version = true;
///                 }
///                 Some("--") => break,
///                 _ => return Err(CliError::Unknown(s)),
///             }
///         }
///
///         Ok(Self { help, version })
///     }
/// }
///
/// let args: Args = onlyargs::parse()?;
///
/// // Returns a string like "onlyargs v0.1.0"
/// assert_eq!(Args::version(), format!("onlyargs v{}\n", env!("CARGO_PKG_VERSION")));
///
/// // Print the help text and exit the process when `--help` is passed to the application.
/// if args.help {
///     args.show_help_and_exit();
/// }
/// # Ok::<(), CliError>(())
/// ```
pub fn parse<T: OnlyArgs>() -> Result<T, CliError> {
    T::parse(env::args_os().skip(1).collect())
}
