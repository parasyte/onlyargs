use myn::prelude::*;
use proc_macro::{Delimiter, Ident, Literal, TokenStream};

#[derive(Debug)]
pub(crate) struct ArgumentStruct {
    pub(crate) name: Ident,
    pub(crate) flags: Vec<ArgFlag>,
    pub(crate) options: Vec<ArgOption>,
    pub(crate) positional: Option<ArgOption>,
    pub(crate) doc: Vec<String>,
}

#[derive(Debug)]
pub(crate) enum Argument {
    Flag(ArgFlag),
    Option(ArgOption),
}

#[derive(Debug)]
pub(crate) struct ArgFlag {
    pub(crate) name: Ident,
    pub(crate) short: Option<char>,
    pub(crate) doc: Vec<String>,
    pub(crate) output: bool,
}

#[derive(Debug)]
pub(crate) struct ArgOption {
    pub(crate) name: Ident,
    pub(crate) short: Option<char>,
    pub(crate) ty_help: ArgType,
    pub(crate) doc: Vec<String>,
    pub(crate) default: Option<Literal>,
    pub(crate) optional: bool,
    pub(crate) positional: bool,
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct ArgView<'a> {
    pub(crate) name: &'a Ident,
    pub(crate) short: Option<char>,
    pub(crate) ty_help: Option<ArgType>,
    pub(crate) doc: &'a [String],
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum ArgType {
    Number,
    OsString,
    Path,
    String,
}

impl ArgumentStruct {
    pub(crate) fn parse(input: TokenStream) -> Result<Self, TokenStream> {
        let mut input = input.into_token_iter();
        let attrs = input.parse_attributes()?;
        input.parse_visibility()?;
        input.expect_ident("struct")?;

        let name = input.as_ident()?;
        let content = input.expect_group(Delimiter::Brace)?;
        let fields = Argument::parse(content)?;

        let mut flags = vec![];
        let mut options = vec![];
        let mut positional = None;

        for field in fields {
            match field {
                Argument::Flag(flag) => flags.push(flag),
                Argument::Option(opt) => match (opt.positional, &positional) {
                    (true, None) => positional = Some(opt),
                    (true, Some(_)) => {
                        return Err(spanned_error(
                            "Positional arguments can only be specified once.",
                            opt.name.span(),
                        ));
                    }
                    _ => options.push(opt),
                },
            }
        }

        let doc = get_doc_comment(&attrs);

        match input.next() {
            None => Ok(Self {
                name,
                flags,
                options,
                positional,
                doc,
            }),
            tree => Err(spanned_error("Unexpected token", tree.as_span())),
        }
    }
}

impl Argument {
    fn parse(mut input: TokenIter) -> Result<Vec<Self>, TokenStream> {
        let mut args = vec![];

        while input.peek().is_some() {
            let attrs = input.parse_attributes()?;

            // Parse attributes
            let doc = get_doc_comment(&attrs);
            let mut default = None;
            let mut long = false;
            let mut short = None;

            for mut attr in attrs {
                let name = attr.name.to_string();
                match name.as_str() {
                    "default" => {
                        let mut stream = attr.tree.expect_group(Delimiter::Parenthesis)?;

                        default = Some(stream.as_lit()?);
                    }
                    "long" => long = true,
                    "short" => {
                        let mut stream = attr.tree.expect_group(Delimiter::Parenthesis)?;
                        let lit = stream.as_lit()?;

                        short = Some(lit.as_char()?);
                    }
                    _ => (),
                }
            }

            input.parse_visibility()?;
            let name = input.as_ident()?;
            input.expect_punct(':')?;
            let (path, span) = input.parse_path()?;
            let _ = input.expect_punct(',');

            let short = if long {
                None
            } else {
                short.or_else(|| {
                    // TODO: Add an attribute to disable short names
                    name.to_string().chars().find(char::is_ascii_alphabetic)
                })
            };

            if path == "bool" {
                args.push(Self::Flag(ArgFlag::new(name, short, doc)));
            } else {
                let mut opt = ArgOption::new(name, short, doc, &path).map_err(|_| {
                    spanned_error(
                        "Expected bool, PathBuf, String, OsString, integer, or float",
                        span,
                    )
                })?;

                opt.default = default;
                if let Some(default) = opt.default.as_ref() {
                    opt.optional = false;
                    let default = default.to_string();
                    if let Some(line) = opt.doc.last_mut() {
                        line.push_str(&format!(" [default: {default}]"));
                    } else {
                        opt.doc.push(format!("[default: {default}]"));
                    }
                } else if !opt.optional {
                    if let Some(line) = opt.doc.last_mut() {
                        line.push_str(" [required]");
                    } else {
                        opt.doc.push("[required]".to_string());
                    }
                }

                args.push(Self::Option(opt));
            }
        }

        Ok(args)
    }
}

impl ArgFlag {
    fn new(name: Ident, short: Option<char>, doc: Vec<String>) -> Self {
        ArgFlag {
            name,
            short,
            doc,
            output: true,
        }
    }

    pub(crate) fn as_view(&self) -> ArgView {
        ArgView {
            name: &self.name,
            short: self.short,
            ty_help: None,
            doc: &self.doc,
        }
    }
}

// We have to check multiple possible paths for types that are not included in
// `std::prelude`. The type system is not available here, so we need to make some educated
// guesses about field types.
const REQUIRED_PATHS: [&str; 4] = [
    "::std::path::PathBuf",
    "std::path::PathBuf",
    "path::PathBuf",
    "PathBuf",
];
const REQUIRED_OS_STRINGS: [&str; 4] = [
    "::std::ffi::OsString",
    "std::ffi::OsString",
    "ffi::OsString",
    "OsString",
];
const REQUIRED_NUMBERS: [&str; 14] = [
    "f32", "f64", "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128",
    "usize",
];
const POSITIONAL_PATHS: [&str; 4] = [
    "Vec<::std::path::PathBuf>",
    "Vec<std::path::PathBuf>",
    "Vec<path::PathBuf>",
    "Vec<PathBuf>",
];
const POSITIONAL_OS_STRINGS: [&str; 4] = [
    "Vec<::std::ffi::OsString>",
    "Vec<std::ffi::OsString>",
    "Vec<ffi::OsString>",
    "Vec<OsString>",
];
const POSITIONAL_NUMBERS: [&str; 14] = [
    "Vec<f32>",
    "Vec<f64>",
    "Vec<i8>",
    "Vec<i16>",
    "Vec<i32>",
    "Vec<i64>",
    "Vec<i128>",
    "Vec<isize>",
    "Vec<u8>",
    "Vec<u16>",
    "Vec<u32>",
    "Vec<u64>",
    "Vec<u128>",
    "Vec<usize>",
];
const OPTIONAL_PATHS: [&str; 4] = [
    "Option<::std::path::PathBuf>",
    "Option<std::path::PathBuf>",
    "Option<path::PathBuf>",
    "Option<PathBuf>",
];
const OPTIONAL_OS_STRINGS: [&str; 4] = [
    "Option<::std::ffi::OsString>",
    "Option<std::ffi::OsString>",
    "Option<ffi::OsString>",
    "Option<OsString>",
];
const OPTIONAL_NUMBERS: [&str; 14] = [
    "Option<f32>",
    "Option<f64>",
    "Option<i8>",
    "Option<i16>",
    "Option<i32>",
    "Option<i64>",
    "Option<i128>",
    "Option<isize>",
    "Option<u8>",
    "Option<u16>",
    "Option<u32>",
    "Option<u64>",
    "Option<u128>",
    "Option<usize>",
];

impl ArgOption {
    fn new(name: Ident, short: Option<char>, doc: Vec<String>, path: &str) -> Result<Self, ()> {
        let optional = if OPTIONAL_PATHS.contains(&path)
            || OPTIONAL_OS_STRINGS.contains(&path)
            || OPTIONAL_NUMBERS.contains(&path)
            || path == "Option<String>"
            || POSITIONAL_PATHS.contains(&path)
            || POSITIONAL_OS_STRINGS.contains(&path)
            || POSITIONAL_NUMBERS.contains(&path)
            || path == "Vec<String>"
        {
            true
        } else if REQUIRED_PATHS.contains(&path)
            || REQUIRED_OS_STRINGS.contains(&path)
            || REQUIRED_NUMBERS.contains(&path)
            || path == "String"
        {
            false
        } else {
            return Err(());
        };

        let ty_help = if OPTIONAL_PATHS.contains(&path)
            || REQUIRED_PATHS.contains(&path)
            || POSITIONAL_PATHS.contains(&path)
        {
            ArgType::Path
        } else if OPTIONAL_OS_STRINGS.contains(&path)
            || REQUIRED_OS_STRINGS.contains(&path)
            || POSITIONAL_OS_STRINGS.contains(&path)
        {
            ArgType::OsString
        } else if path == "String" || path == "Vec<String>" || path == "Option<String>" {
            ArgType::String
        } else if OPTIONAL_NUMBERS.contains(&path)
            || REQUIRED_NUMBERS.contains(&path)
            || POSITIONAL_NUMBERS.contains(&path)
        {
            ArgType::Number
        } else {
            unreachable!();
        };

        let positional = POSITIONAL_PATHS.contains(&path)
            || POSITIONAL_OS_STRINGS.contains(&path)
            || POSITIONAL_NUMBERS.contains(&path)
            || path == "Vec<String>";

        Ok(ArgOption {
            name,
            short,
            ty_help,
            doc,
            default: None,
            optional,
            positional,
        })
    }

    pub(crate) fn as_view(&self) -> ArgView {
        ArgView {
            name: &self.name,
            short: self.short,
            ty_help: Some(self.ty_help),
            doc: &self.doc,
        }
    }
}

impl ArgType {
    pub(crate) fn as_str(&self) -> &str {
        match self {
            Self::Number => " NUMBER",
            Self::OsString | Self::String => " STRING",
            Self::Path => " PATH",
        }
    }
}
