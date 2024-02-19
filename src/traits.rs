use crate::CliError;
use std::ffi::OsString;
use std::num::{ParseFloatError, ParseIntError};
use std::path::PathBuf;
use std::str::FromStr;

/// An extension trait for `Option<OsString>` that provides some parsers that are useful for CLIs.
pub trait ArgExt {
    /// Parse an argument into a `String`.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the argument is `None` or not valid UTF-8.
    fn parse_str<N>(self, name: N) -> Result<String, CliError>
    where
        N: Into<String>;

    /// Parse an argument into a `PathBuf`.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the argument is `None`.
    fn parse_path<N>(self, name: N) -> Result<PathBuf, CliError>
    where
        N: Into<String>;

    /// Parse an argument into an `OsString`.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the argument is `None`.
    fn parse_osstr<N>(self, name: N) -> Result<OsString, CliError>
    where
        N: Into<String>;

    /// Parse an argument into a primitive integer.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the argument is `None` or not a valid integer.
    fn parse_int<T, N>(self, name: N) -> Result<T, CliError>
    where
        N: Into<String>,
        T: FromStr<Err = ParseIntError>;

    /// Parse an argument into a primitive floating point number.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the argument is `None` or not valid floating point number.
    fn parse_float<T, N>(self, name: N) -> Result<T, CliError>
    where
        N: Into<String>,
        T: FromStr<Err = ParseFloatError>;
}

/// An extension trait for required arguments.
pub trait RequiredArgExt {
    /// The inner type that the trait methods return.
    ///
    /// For `Option<T>`, this would be `type Inner = T;`.
    type Inner;

    /// Unwrap an argument that is required by the CLI.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the argument is `None`.
    fn required<N>(self, name: N) -> Result<Self::Inner, CliError>
    where
        N: Into<String>;
}

impl ArgExt for Option<OsString> {
    fn parse_str<N>(self, name: N) -> Result<String, CliError>
    where
        N: Into<String>,
    {
        let name = name.into();
        self.ok_or_else(|| CliError::MissingValue(name.clone()))?
            .into_string()
            .map_err(|err| CliError::ParseStrError(name, err))
    }

    fn parse_path<N>(self, name: N) -> Result<PathBuf, CliError>
    where
        N: Into<String>,
    {
        Ok(self
            .ok_or_else(|| CliError::MissingValue(name.into()))?
            .into())
    }

    fn parse_osstr<N>(self, name: N) -> Result<OsString, CliError>
    where
        N: Into<String>,
    {
        self.ok_or_else(|| CliError::MissingValue(name.into()))
    }

    fn parse_int<T, N>(self, name: N) -> Result<T, CliError>
    where
        N: Into<String>,
        T: FromStr<Err = ParseIntError>,
    {
        let name = name.into();

        self.clone().parse_str(&name).and_then(|string| {
            string
                .parse::<T>()
                .map_err(|err| CliError::ParseIntError(name, self.unwrap(), err))
        })
    }

    fn parse_float<T, N>(self, name: N) -> Result<T, CliError>
    where
        N: Into<String>,
        T: FromStr<Err = ParseFloatError>,
    {
        let name = name.into();

        self.clone().parse_str(&name).and_then(|string| {
            string
                .parse::<T>()
                .map_err(|err| CliError::ParseFloatError(name, self.unwrap(), err))
        })
    }
}

impl ArgExt for OsString {
    fn parse_str<N>(self, name: N) -> Result<String, CliError>
    where
        N: Into<String>,
    {
        let name = name.into();
        self.into_string()
            .map_err(|err| CliError::ParseStrError(name, err))
    }

    fn parse_path<N>(self, _name: N) -> Result<PathBuf, CliError>
    where
        N: Into<String>,
    {
        Ok(self.into())
    }

    fn parse_osstr<N>(self, _name: N) -> Result<OsString, CliError>
    where
        N: Into<String>,
    {
        Ok(self)
    }

    fn parse_int<T, N>(self, name: N) -> Result<T, CliError>
    where
        N: Into<String>,
        T: FromStr<Err = ParseIntError>,
    {
        let name = name.into();

        self.clone().parse_str(&name).and_then(|string| {
            string
                .parse::<T>()
                .map_err(|err| CliError::ParseIntError(name, self, err))
        })
    }

    fn parse_float<T, N>(self, name: N) -> Result<T, CliError>
    where
        N: Into<String>,
        T: FromStr<Err = ParseFloatError>,
    {
        let name = name.into();

        self.clone().parse_str(&name).and_then(|string| {
            string
                .parse::<T>()
                .map_err(|err| CliError::ParseFloatError(name, self, err))
        })
    }
}

impl<T> RequiredArgExt for Option<T> {
    type Inner = T;

    fn required<N>(self, name: N) -> Result<Self::Inner, CliError>
    where
        N: Into<String>,
    {
        self.ok_or_else(|| CliError::MissingRequired(name.into()))
    }
}

impl<T> RequiredArgExt for Vec<T> {
    type Inner = Vec<T>;

    fn required<N>(self, name: N) -> Result<Self::Inner, CliError>
    where
        N: Into<String>,
    {
        if self.is_empty() {
            Err(CliError::MissingRequired(name.into()))
        } else {
            Ok(self)
        }
    }
}
