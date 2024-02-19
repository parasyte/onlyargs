use myn::prelude::*;
use proc_macro::{Delimiter, Ident, Literal, Span, TokenStream};

#[derive(Debug)]
pub(crate) struct ArgumentStruct {
    pub(crate) name: Ident,
    pub(crate) flags: Vec<ArgFlag>,
    pub(crate) options: Vec<ArgOption>,
    pub(crate) positional: Option<ArgOption>,
    pub(crate) doc: Vec<String>,
    pub(crate) footer: Vec<String>,
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
    pub(crate) default: bool,
    pub(crate) output: bool,
}

#[derive(Debug)]
pub(crate) struct ArgOption {
    pub(crate) name: Ident,
    pub(crate) short: Option<char>,
    pub(crate) ty_help: ArgType,
    pub(crate) doc: Vec<String>,
    pub(crate) default: Option<Literal>,
    pub(crate) property: ArgProperty,
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
    Float,
    Integer,
    OsString,
    Path,
    String,
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum ArgProperty {
    Required,
    Optional,
    MultiValue { required: bool },
    Positional { required: bool },
}

impl ArgumentStruct {
    pub(crate) fn parse(input: TokenStream) -> Result<Self, TokenStream> {
        let mut input = input.into_token_iter();
        let attrs = input.parse_attributes()?;
        input.parse_visibility()?;
        input.expect_ident("struct")?;

        let name = input.try_ident()?;
        let content = input.expect_group(Delimiter::Brace)?;
        let fields = Argument::parse(content)?;

        let mut flags = vec![];
        let mut options = vec![];
        let mut positional = None;

        for field in fields {
            match field {
                Argument::Flag(flag) => flags.push(flag),
                Argument::Option(opt) => match (opt.property, &positional) {
                    (ArgProperty::Positional { .. }, None) => positional = Some(opt),
                    (ArgProperty::Positional { .. }, Some(_)) => {
                        return Err(spanned_error(
                            "Positional arguments can only be specified once.",
                            opt.name.span(),
                        ));
                    }
                    _ => options.push(opt),
                },
            }
        }

        let doc = get_doc_comment(&attrs)
            .into_iter()
            .map(trim_with_indent)
            .collect();

        let footer = get_attr_strings(&attrs, "footer")
            .into_iter()
            .map(|line| line.trim_end().to_string())
            .collect();

        match input.next() {
            None => Ok(Self {
                name,
                flags,
                options,
                positional,
                doc,
                footer,
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
            let doc = get_doc_comment(&attrs)
                .into_iter()
                .map(trim_with_indent)
                .collect();
            let mut default = None;
            let mut long = false;
            let mut short = None;
            let mut required = false;
            let mut positional = false;

            for mut attr in attrs {
                let name = attr.name.to_string();
                match name.as_str() {
                    "default" => {
                        let mut stream = attr.tree.expect_group(Delimiter::Parenthesis)?;

                        default = Some(stream.try_lit().or_else(|_| {
                            stream
                                .try_ident()
                                .and_then(|ident| match ident.to_string().as_str() {
                                    boolean @ ("true" | "false") => Ok(Literal::string(boolean)),
                                    _ => Err(spanned_error("Unexpected identifier", ident.span())),
                                })
                        })?);
                    }
                    "long" => long = true,
                    "positional" => positional = true,
                    "required" => required = true,
                    "short" => {
                        let mut stream = attr.tree.expect_group(Delimiter::Parenthesis)?;
                        let lit = stream.try_lit()?;

                        short = Some(lit.as_char()?);
                    }
                    _ => (),
                }
            }

            input.parse_visibility()?;
            let name = input.try_ident()?;
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
                if required {
                    return Err(spanned_error(
                        "#[required] can only be used on `Vec<T>`",
                        span,
                    ));
                }
                if positional {
                    return Err(spanned_error(
                        "#[positional] can only be used on `Vec<T>`",
                        span,
                    ));
                }

                let mut flag = ArgFlag::new(name, short, doc);
                match default {
                    Some(lit) if lit.to_string() == r#""true""# => flag.default = true,
                    _ => (),
                }
                args.push(Self::Flag(flag));
            } else {
                let mut opt = ArgOption::new(span, name, short, doc, &path)?;

                apply_default(span, &mut opt, default)?;
                apply_required(span, &mut opt, required)?;
                apply_positional(span, &mut opt, positional)?;

                if let Some(default) = opt.default.as_ref() {
                    let default = default.to_string();
                    if let Some(line) = opt.doc.last_mut() {
                        line.push_str(&format!(" [default: {default}]"));
                    } else {
                        opt.doc.push(format!("[default: {default}]"));
                    }
                } else if matches!(
                    opt.property,
                    ArgProperty::Required
                        | ArgProperty::Positional { required: true }
                        | ArgProperty::MultiValue { required: true }
                ) {
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

fn apply_default(
    span: Span,
    opt: &mut ArgOption,
    default: Option<Literal>,
) -> Result<(), TokenStream> {
    match (default.is_some(), &opt.property) {
        (true, ArgProperty::Required) => opt.default = default,
        (true, _) => {
            return Err(spanned_error(
                "#[default(...)] can only be used on primitive types",
                span,
            ));
        }
        (false, _) => (),
    }

    Ok(())
}

fn apply_required(span: Span, opt: &mut ArgOption, required: bool) -> Result<(), TokenStream> {
    match (required, &mut opt.property) {
        (false, _) => (),
        (true, ArgProperty::MultiValue { required }) => *required = true,
        _ => {
            return Err(spanned_error(
                "#[required] can only be used on `Vec<T>`",
                span,
            ));
        }
    }

    Ok(())
}

fn apply_positional(span: Span, opt: &mut ArgOption, positional: bool) -> Result<(), TokenStream> {
    match (positional, &opt.property) {
        (true, ArgProperty::MultiValue { required }) => {
            opt.property = ArgProperty::Positional {
                required: *required,
            }
        }
        (true, _) => {
            return Err(spanned_error(
                "#[positional] can only be used on `Vec<T>`",
                span,
            ));
        }
        (false, _) => (),
    }

    Ok(())
}

impl ArgFlag {
    fn new(name: Ident, short: Option<char>, doc: Vec<String>) -> Self {
        ArgFlag {
            name,
            short,
            doc,
            default: false,
            output: true,
        }
    }

    pub(crate) fn new_priv(name: Ident, short: Option<char>, doc: Vec<String>) -> Self {
        ArgFlag {
            name,
            short,
            doc,
            default: false,
            output: false,
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
const REQUIRED_FLOATS: [&str; 2] = ["f32", "f64"];
const REQUIRED_INTEGERS: [&str; 12] = [
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize",
];
const MULTI_PATHS: [&str; 4] = [
    "Vec<::std::path::PathBuf>",
    "Vec<std::path::PathBuf>",
    "Vec<path::PathBuf>",
    "Vec<PathBuf>",
];
const MULTI_OS_STRINGS: [&str; 4] = [
    "Vec<::std::ffi::OsString>",
    "Vec<std::ffi::OsString>",
    "Vec<ffi::OsString>",
    "Vec<OsString>",
];
const MULTI_FLOATS: [&str; 2] = ["Vec<f32>", "Vec<f64>"];
const MULTI_INTEGERS: [&str; 12] = [
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
const OPTIONAL_FLOATS: [&str; 2] = ["Option<f32>", "Option<f64>"];
const OPTIONAL_INTEGERS: [&str; 12] = [
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
    fn new(
        span: Span,
        name: Ident,
        short: Option<char>,
        doc: Vec<String>,
        path: &str,
    ) -> Result<Self, TokenStream> {
        // Parse the argument type and decide what properties it should start with.
        let property = if OPTIONAL_PATHS.contains(&path)
            || OPTIONAL_OS_STRINGS.contains(&path)
            || OPTIONAL_FLOATS.contains(&path)
            || OPTIONAL_INTEGERS.contains(&path)
            || path == "Option<String>"
        {
            ArgProperty::Optional
        } else if MULTI_PATHS.contains(&path)
            || MULTI_OS_STRINGS.contains(&path)
            || MULTI_FLOATS.contains(&path)
            || MULTI_INTEGERS.contains(&path)
            || path == "Vec<String>"
        {
            ArgProperty::MultiValue { required: false }
        } else if REQUIRED_PATHS.contains(&path)
            || REQUIRED_OS_STRINGS.contains(&path)
            || REQUIRED_FLOATS.contains(&path)
            || REQUIRED_INTEGERS.contains(&path)
            || path == "String"
        {
            ArgProperty::Required
        } else {
            return Err(spanned_error(
                "Expected bool, PathBuf, String, OsString, integer, or float",
                span,
            ));
        };

        // Decide the type to show in the help message.
        let ty_help = if OPTIONAL_PATHS.contains(&path)
            || REQUIRED_PATHS.contains(&path)
            || MULTI_PATHS.contains(&path)
        {
            ArgType::Path
        } else if OPTIONAL_OS_STRINGS.contains(&path)
            || REQUIRED_OS_STRINGS.contains(&path)
            || MULTI_OS_STRINGS.contains(&path)
        {
            ArgType::OsString
        } else if path == "String" || path == "Vec<String>" || path == "Option<String>" {
            ArgType::String
        } else if OPTIONAL_FLOATS.contains(&path)
            || REQUIRED_FLOATS.contains(&path)
            || MULTI_FLOATS.contains(&path)
        {
            ArgType::Float
        } else if OPTIONAL_INTEGERS.contains(&path)
            || REQUIRED_INTEGERS.contains(&path)
            || MULTI_INTEGERS.contains(&path)
        {
            ArgType::Integer
        } else {
            unreachable!();
        };

        Ok(ArgOption {
            name,
            short,
            ty_help,
            doc,
            default: None,
            property,
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
            Self::Float => " FLOAT",
            Self::Integer => " INTEGER",
            Self::OsString | Self::String => " STRING",
            Self::Path => " PATH",
        }
    }

    pub(crate) fn converter(&self) -> &str {
        match self {
            Self::Float | Self::Integer => "",
            Self::OsString | Self::Path | Self::String => ".into()",
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
fn trim_with_indent(line: String) -> String {
    line.strip_prefix(' ')
        .unwrap_or(&line)
        .trim_end()
        .to_string()
}
