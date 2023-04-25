//! Only argument parsing! Nothing more.
//!
//! `onlyargs` is an obsessively tiny argument parsing library. It provides a basic trait and helper
//! functions for parsing arguments from the environment.
//!
//! Implement the [`OnlyArgs`] trait on your own argument type and use any of the parser functions
//! to create your CLI. The trait can also be derived with the [`onlyargs_derive`] crate if you are
//! OK with an opinionated parser and just want to reduce the amount of boilerplate in your code.
//!
//! [`onlyargs_derive`]: https://docs.rs/onlyargs_derive

#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use std::env;
use std::ffi::OsString;
use std::fmt::Display;

pub mod traits;

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

/// The primary argument parser trait.
///
/// This trait can be derived with the [`onlyargs_derive`](https://docs.rs/onlyargs_derive) crate.
///
/// See the [`parse`] function for more information.
pub trait OnlyArgs {
    /// The application help string.
    const HELP: &'static str = concat!(
        env!("CARGO_PKG_NAME"),
        " v",
        env!("CARGO_PKG_VERSION"),
        "\n",
        env!("CARGO_PKG_DESCRIPTION"),
        "\n",
    );

    /// The application name and version.
    const VERSION: &'static str = concat!(
        env!("CARGO_PKG_NAME"),
        " v",
        env!("CARGO_PKG_VERSION"),
        "\n",
    );

    /// Construct a type that implements this trait.
    ///
    /// Each argument is provided as an [`OsString`].
    ///
    /// # Errors
    ///
    /// Returns `Err` if the command line arguments cannot be parsed to `Self`.
    fn parse(args: Vec<OsString>) -> Result<Self, CliError>
    where
        Self: Sized;

    /// Print the application help string and exit the process.
    fn help() -> ! {
        eprintln!("{}", Self::HELP);
        std::process::exit(0);
    }

    /// Print the application name and version and exit the process.
    fn version() -> ! {
        eprintln!("{}", Self::VERSION);
        std::process::exit(0);
    }
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

/// Type constructor for argument parser.
///
/// Given a type that implements [`OnlyArgs`], this function will construct the type from the
/// current environment.
///
/// # Errors
///
/// Returns `Err` if arguments from the environment cannot be parsed to `T`.
///
/// # Example
///
/// ```
/// # use std::ffi::OsString;
/// # use onlyargs::{CliError, OnlyArgs};
/// struct Args {
///     verbose: bool,
/// }
///
/// impl OnlyArgs for Args {
///     fn parse(args: Vec<OsString>) -> Result<Self, CliError> {
///         let mut verbose = false;
///
///         for arg in args.into_iter() {
///             match arg.to_str() {
///                 Some("--help") | Some("-h") => {
///                     Self::help();
///                 }
///                 Some("--version") | Some("-V") => {
///                     Self::version();
///                 }
///                 Some("--verbose") | Some("-v") => {
///                     verbose = true;
///                 }
///                 Some("--") => break,
///                 _ => return Err(CliError::Unknown(arg)),
///             }
///         }
///
///         Ok(Self { verbose })
///     }
/// }
///
/// let args: Args = onlyargs::parse()?;
///
/// // Returns a string like "onlyargs v0.1.0"
/// assert_eq!(Args::VERSION, format!("onlyargs v{}\n", env!("CARGO_PKG_VERSION")));
///
/// if args.verbose {
///     println!("Verbose output is enabled");
/// }
/// # Ok::<(), CliError>(())
/// ```
pub fn parse<T: OnlyArgs>() -> Result<T, CliError> {
    T::parse(env::args_os().skip(1).collect())
}

mod macros {
    #[macro_export]
    macro_rules! impl_help {
        () => {
            concat!(
                env!("CARGO_PKG_NAME"),
                " v",
                env!("CARGO_PKG_VERSION"),
                "\n",
                env!("CARGO_PKG_DESCRIPTION"),
                "\n",
            )
        };
    }

    #[macro_export]
    macro_rules! impl_version {
        () => {
            concat!(
                env!("CARGO_PKG_NAME"),
                " v",
                env!("CARGO_PKG_VERSION"),
                "\n",
            );
        };
    }
}
