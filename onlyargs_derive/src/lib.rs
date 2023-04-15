//! Derive macro for [`onlyargs`](https://docs.rs/onlyargs).
//!
//! The parser generated by this macro is very opinionated. The implementation attempts to be as
//! light as possible while also being usable for most applications.
//!
//! # Example
//!
//! ```
//! use onlyargs_derive::OnlyArgs;
//!
//! /// Doc comments will appear in your application's help text.
//! /// Multi-line comments are also supported.
//! #[derive(Debug, OnlyArgs)]
//! struct Args {
//!     /// Optional output path.
//!     output: Option<std::path::PathBuf>,
//!
//!     /// Enable verbose output.
//!     verbose: bool,
//! }
//!
//! let args: Args = onlyargs::parse()?;
//!
//! if let Some(output) = args.output {
//!     if args.verbose {
//!         eprintln!("Creating file: `{path}`", path = output.display());
//!     }
//!
//!     // Do something with `output`...
//! }
//! # Ok::<_, onlyargs::CliError>(())
//! ```
//!
//! # DSL reference
//!
//! Only structs with named fields are supported. Doc comments are used for the generated help text.
//! Argument names are generated automatically from field names with only a few rules:
//!
//! - Long argument names start with `--`, ASCII alphabetic characters are made lowercase, and all
//!   `_` characters are replaced with `-`.
//! - Short argument names use the first ASCII alphabetic character of the field name following a
//!   `-`. Short arguments are not allowed to be duplicated.
//!   - This behavior can be suppressed with the `#[long]` attribute (see below).
//!   - Alternatively, the `#[short('…')]` attribute can be used to set a specific short name.
//!
//! # Provided arguments
//!
//! `--help|-h` and `--version|-V` arguments are automatically generated. When the parser encounters
//! either, it will print the help or version message and exit the application with exit code 0.
//!
//! # Field attributes
//!
//! Parsing options are configurable with the following attributes:
//!
//! - `#[long]`: Only generate long argument names like `--help`. Short args like `-h` are generated
//!   by default, and this attribute suppresses that behavior.
//! - `#[short('N')]`: Generate a short argument name with the given character. In this example, it
//!   will be `-N`.
//!   - If `#[long]` and `#[short]` are used together, `#[long]` takes precedence.
//! - `#[default(T)]`: Specify a default value for an argument. Where `T` is a literal value.
//!
//! # Supported types
//!
//! Here is the list of supported field "primitive" types:
//!
//! | Type             | Description                                      |
//! |------------------|--------------------------------------------------|
//! | `bool`           | Defines a flag.                                  |
//! | `f32`\|`f64`     | Floating point number option.                    |
//! | `i8`\|`u8`       | 8-bit integer option.                            |
//! | `i16`\|`u16`     | 16-bit integer option.                           |
//! | `i32`\|`u32`     | 32-bit integer option.                           |
//! | `i64`\|`u64`     | 64-bit integer option.                           |
//! | `i128`\|`u128`   | 128-bit integer option.                          |
//! | `isize`\|`usize` | Pointer-sized integer option.                    |
//! | `OsString`       | A string option with platform-specific encoding. |
//! | `PathBuf`        | A file system path option.                       |
//! | `String`         | UTF-8 encoded string option.                     |
//!
//! Additionally, some wrapper and composite types are also available, where the type `T` must be
//! one of the primitive types listed above.
//!
//! | Type        | Description                       |
//! |-------------|-----------------------------------|
//! | `Option<T>` | An optional argument.             |
//! | `Vec<T>`    | Positional arguments (see below). |
//!
//! In argument parsing parlance, "flags" are simple boolean values; the argument does not require
//! a value. For example, the argument `--help`.
//!
//! "Options" carry a value and the argument parser requires the value to directly follow the
//! argument name. Arguments can be made optional with `Option<T>`.
//!
//! ## Positional arguments
//!
//! If the struct contains a field with a vector type, it _must_ be the only vector field. This
//! becomes the "dumping ground" for all positional arguments, which are any args that do not match
//! an existing field, or any arguments following the `--` "stop parsing" sentinel.

#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use crate::parser::{ArgFlag, ArgOption, ArgType, ArgView, ArgumentStruct};
use myn::utils::spanned_error;
use proc_macro::{Ident, Span, TokenStream};
use std::{collections::HashMap, str::FromStr as _};

mod parser;

/// See the [root module documentation](crate) for the DSL specification.
#[allow(clippy::too_many_lines)]
#[proc_macro_derive(OnlyArgs, attributes(default, long, short))]
pub fn derive_parser(input: TokenStream) -> TokenStream {
    let ast = match ArgumentStruct::parse(input) {
        Ok(ast) => ast,
        Err(err) => return err,
    };

    let mut flags = vec![
        ArgFlag {
            name: Ident::new("help", Span::call_site()),
            short: Some('h'),
            doc: vec!["Show this help message.".to_string()],
            output: false,
        },
        ArgFlag {
            name: Ident::new("version", Span::call_site()),
            short: Some('V'),
            doc: vec!["Show the application version.".to_string()],
            output: false,
        },
    ];
    flags.extend(ast.flags.into_iter());

    // De-dupe short args.
    let mut dupes = HashMap::new();
    for flag in &flags {
        if let Err(err) = dedupe(&mut dupes, flag.as_view()) {
            return err;
        }
    }
    for opt in &ast.options {
        if let Err(err) = dedupe(&mut dupes, opt.as_view()) {
            return err;
        }
    }

    // Produce help text for all arguments.
    let max_width = get_max_width(flags.iter().map(ArgFlag::as_view));
    let flags_help = flags
        .iter()
        .map(|arg| to_help(arg.as_view(), max_width))
        .collect::<String>();

    let max_width = get_max_width(ast.options.iter().map(ArgOption::as_view));
    let options_help = ast
        .options
        .iter()
        .map(|arg| to_help(arg.as_view(), max_width))
        .collect::<String>();

    let positional_header = ast
        .positional
        .as_ref()
        .map(|opt| format!(" [{}...]", opt.name))
        .unwrap_or_default();
    let positional_help = ast
        .positional
        .as_ref()
        .map(|opt| format!("\n{}:\n  {}", opt.name, opt.doc.join("\n  ")))
        .unwrap_or_default();

    // Produce variables for argument parser state.
    let flags_vars = flags
        .iter()
        .filter_map(|flag| {
            flag.output.then(|| {
                let name = &flag.name;
                format!("let mut {name} = false;")
            })
        })
        .collect::<String>();
    let options_vars = ast
        .options
        .iter()
        .map(|opt| {
            let name = &opt.name;
            if let Some(default) = opt.default.as_ref() {
                format!("let mut {name} = {default};")
            } else {
                format!("let mut {name} = None;")
            }
        })
        .collect::<String>();
    let positional_var = ast
        .positional
        .as_ref()
        .map(|opt| {
            let name = &opt.name;
            format!("let mut {name} = vec![];")
        })
        .unwrap_or_default();

    // Produce matchers for parser.
    let flags_matchers = flags
        .iter()
        .filter_map(|flag| {
            flag.output.then(|| {
                let name = &flag.name;
                let short = flag
                    .short
                    .map(|ch| format!(r#"| Some("-{ch}")"#))
                    .unwrap_or_default();

                format!(
                    r#"Some("--{arg}") {short} => {name} = true,"#,
                    arg = to_arg_name(name)
                )
            })
        })
        .collect::<String>();
    let options_matchers = ast
        .options
        .iter()
        .map(|opt| {
            let name = &opt.name;
            let short = opt
                .short
                .map(|ch| format!(r#"| Some(name @ "-{ch}")"#))
                .unwrap_or_default();
            let value = if opt.default.is_some() {
                match opt.ty_help {
                    ArgType::Number => "args.next().parse_int(name)?",
                    ArgType::OsString => "args.next().parse_osstr(name)?",
                    ArgType::Path => "args.next().parse_path(name)?",
                    ArgType::String => "args.next().parse_str(name)?",
                }
            } else {
                match opt.ty_help {
                    ArgType::Number => "Some(args.next().parse_int(name)?)",
                    ArgType::OsString => "Some(args.next().parse_osstr(name)?)",
                    ArgType::Path => "Some(args.next().parse_path(name)?)",
                    ArgType::String => "Some(args.next().parse_str(name)?)",
                }
            };

            format!(
                r#"Some(name @ "--{arg}") {short} => {name} = {value},"#,
                arg = to_arg_name(name)
            )
        })
        .collect::<String>();
    let positional_matcher = match ast.positional.as_ref() {
        Some(opt) => {
            let name = &opt.name;
            let value = match opt.ty_help {
                ArgType::Number => r#"arg.parse_int("<POSITIONAL>")?"#,
                ArgType::OsString => r#"arg.parse_osstr("<POSITIONAL>")?"#,
                ArgType::Path => r#"arg.parse_path("<POSITIONAL>")?"#,
                ArgType::String => r#"arg.parse_str("<POSITIONAL>")?"#,
            };

            format!(
                r#"
                    Some("--") => {{
                        for arg in args {{
                            {name}.push({value});
                        }}
                        break;
                    }}
                    _ => {name}.push({value}),
                "#
            )
        }
        None => r#"
            Some("--") => break,
            _ => return Err(::onlyargs::CliError::Unknown(arg)),
        "#
        .to_string(),
    };

    // Produce identifiers for args constructor.
    let flags_idents = flags
        .iter()
        .filter_map(|flag| flag.output.then_some(format!("{},", flag.name)))
        .collect::<String>();
    let options_idents = ast
        .options
        .iter()
        .map(|opt| {
            let name = &opt.name;
            if opt.default.is_some() || opt.optional {
                format!("{name},")
            } else {
                format!(
                    r#"{name}: {name}.required("--{arg}")?,"#,
                    arg = to_arg_name(name)
                )
            }
        })
        .collect::<String>();
    let positional_ident = ast
        .positional
        .map(|opt| format!("{},", opt.name))
        .unwrap_or_default();

    let name = ast.name;
    let doc_comment = if ast.doc.is_empty() {
        String::new()
    } else {
        format!("\n{}\n", ast.doc.join("\n"))
    };
    let bin_name = std::env::var_os("CARGO_BIN_NAME").and_then(|name| name.into_string().ok());
    let help_impl = if bin_name.is_none() {
        r#"fn help() -> ! {
            let bin_name = ::std::env::args_os()
                .next()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            ::std::eprintln!("{}", Self::HELP.replace("{bin_name}", &bin_name));
            ::std::process::exit(0);
        }"#
    } else {
        ""
    };
    let bin_name = bin_name.unwrap_or_else(|| "{bin_name}".to_string());

    // Produce final code.
    let code = TokenStream::from_str(&format!(
        r#"
            impl ::onlyargs::OnlyArgs for {name} {{
                const HELP: &'static str = ::std::concat!(
                    env!("CARGO_PKG_NAME"),
                    " v",
                    env!("CARGO_PKG_VERSION"),
                    "\n",
                    env!("CARGO_PKG_DESCRIPTION"),
                    "\n",
                    {doc_comment:?},
                    "\nUsage:\n  ",
                    {bin_name:?},
                    " [flags] [options]",
                    {positional_header:?},
                    "\n\nFlags:\n",
                    {flags_help:?},
                    "\nOptions:\n",
                    {options_help:?},
                    {positional_help:?},
                    "\n",
                );

                {help_impl}

                fn parse(args: Vec<::std::ffi::OsString>) ->
                    ::std::result::Result<Self, ::onlyargs::CliError>
                {{
                    use ::onlyargs::traits::*;
                    use ::std::option::Option::{{None, Some}};
                    use ::std::result::Result::{{Err, Ok}};

                    {flags_vars}
                    {options_vars}
                    {positional_var}

                    let mut args = args.into_iter();
                    while let Some(arg) = args.next() {{
                        match arg.to_str() {{
                            // TODO: Add an attribute to disable help/version.
                            Some("--help") | Some("-h") => Self::help(),
                            Some("--version") | Some("-V") => Self::version(),
                            {flags_matchers}
                            {options_matchers}
                            {positional_matcher}
                        }}
                    }}

                    Ok(Self {{
                        {flags_idents}
                        {options_idents}
                        {positional_ident}
                    }})
                }}
            }}
        "#
    ));

    match code {
        Ok(stream) => stream,
        Err(err) => spanned_error(err.to_string(), Span::call_site()),
    }
}

// 1 hyphen + 1 char + 1 trailing space.
const SHORT_PAD: usize = 3;
// 2 leading spaces + 2 hyphens + 2 trailing spaces.
const LONG_PAD: usize = 6;

fn to_arg_name(ident: &Ident) -> String {
    let mut name = ident.to_string().replace('_', "-");
    name.make_ascii_lowercase();

    name
}

fn to_help(view: ArgView, max_width: usize) -> String {
    let name = to_arg_name(view.name);
    let ty = match view.ty_help.as_ref() {
        Some(ty_help) => ty_help.as_str(),
        None => "",
    };
    let pad = " ".repeat(max_width + LONG_PAD);
    let help = view.doc.join(&format!("\n{pad}"));

    let width = max_width - name.len();
    if let Some(ch) = view.short {
        let width = width - SHORT_PAD;

        format!("  -{ch} --{name}{ty:<width$}  {help}\n")
    } else {
        format!("  --{name}{ty:<width$}  {help}\n")
    }
}

fn get_max_width<'a, I>(iter: I) -> usize
where
    I: Iterator<Item = ArgView<'a>>,
{
    iter.fold(0, |acc, view| {
        let short = view.short.map(|_| SHORT_PAD).unwrap_or_default();
        let ty = match view.ty_help.as_ref() {
            Some(ty_help) => ty_help.as_str(),
            None => "",
        };

        acc.max(view.name.to_string().len() + ty.len() + short)
    })
}

fn dedupe<'a>(dupes: &mut HashMap<char, &'a Ident>, arg: ArgView<'a>) -> Result<(), TokenStream> {
    if let Some(ch) = arg.short {
        if let Some(other) = dupes.get(&ch) {
            let msg =
                format!("Only one short arg is allowed. `-{ch}` also used on field `{other}`");

            return Err(spanned_error(msg, arg.name.span()));
        }

        dupes.insert(ch, arg.name);
    }

    Ok(())
}
