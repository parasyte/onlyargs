use proc_macro2::{Ident, Literal, Span, TokenStream, TokenTree};
use venial::{Attribute, AttributeValue, Declaration, NamedField, StructFields};

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
    #[allow(clippy::manual_let_else)]
    pub(crate) fn parse(input: TokenStream) -> Result<Self, venial::Error> {
        let decl = venial::parse_declaration(input)?;

        let s = match decl {
            Declaration::Struct(s) => s,
            _ => return Err(venial::Error::new("TODO: Invalid decl")),
        };

        let name = s.name;
        let mut fields = vec![];
        match s.fields {
            StructFields::Named(f) => {
                for arg in f.fields.items().cloned().map(Argument::parse) {
                    fields.push(arg?);
                }
            }
            _ => return Err(venial::Error::new("TODO: Invalid struct fields")),
        }

        // TODO: Validate attributes and merge with argument

        let mut flags = vec![];
        let mut options = vec![];
        let mut positional = None;

        for field in fields {
            match field {
                Argument::Flag(flag) => flags.push(flag),
                Argument::Option(opt) => match (opt.positional, &positional) {
                    (true, None) => positional = Some(opt),
                    (true, Some(_)) => {
                        return Err(venial::Error::new_at_span(
                            opt.name.span(),
                            "Positional arguments can only be specified once.",
                        ));
                    }
                    _ => options.push(opt),
                },
            }
        }

        let doc = get_doc_comment(&s.attributes);

        Ok(Self {
            name,
            flags,
            options,
            positional,
            doc,
        })
    }
}

fn get_doc_comment(attrs: &[Attribute]) -> Vec<String> {
    attrs
        .iter()
        .map(|a| a.path.iter().map(ToString::to_string).collect::<String>())
        .collect()
}

impl Argument {
    fn parse(field: NamedField) -> Result<Self, venial::Error> {
        // Parse attributes
        let doc = get_doc_comment(&field.attributes);
        let mut default = None;
        let mut long = false;
        let mut short = None;

        for attr in field.attributes {
            let name = attr
                .path
                .into_iter()
                .map(|p| p.to_string())
                .collect::<String>();

            match name.as_str() {
                "default" => {
                    // There must be a better way!
                    let lit = match attr.value {
                        AttributeValue::Group(_, tree) => {
                            let tree = tree
                                .into_iter()
                                .next()
                                .ok_or_else(|| venial::Error::new("TODO: Expected TokenTree"))?;
                            match tree {
                                proc_macro2::TokenTree::Literal(lit) => lit,
                                _ => return Err(venial::Error::new("TODO: Expected Literal")),
                            }
                        }
                        _ => return Err(venial::Error::new("TODO: Unexpected value")),
                    };

                    default = Some(lit);
                }
                "long" => long = true,
                "short" => {
                    // There must be a better way!
                    let lit = match attr.value {
                        AttributeValue::Group(_, tree) => {
                            tree.into_iter().map(|t| t.to_string()).collect::<String>()
                        }
                        _ => return Err(venial::Error::new("TODO: Unexpected value")),
                    };

                    let ch = lit
                        .chars()
                        .nth(1)
                        .ok_or_else(|| venial::Error::new("TODO: Invalid char"))?;

                    short = Some(ch);
                }
                _ => (),
            }
        }

        let name = field.name;
        let path = field
            .ty
            .tokens
            .iter()
            .map(ToString::to_string)
            .collect::<String>();
        let span = field
            .ty
            .tokens
            .first()
            .map_or_else(Span::call_site, TokenTree::span);

        let short = if long {
            None
        } else {
            short.or_else(|| {
                // TODO: Add an attribute to disable short names
                name.to_string().chars().find(char::is_ascii_alphabetic)
            })
        };

        if path == "bool" {
            Ok(Self::Flag(ArgFlag::new(name, short, doc)))
        } else {
            let mut opt = ArgOption::new(name, short, doc, &path).map_err(|_| {
                venial::Error::new_at_span(
                    span,
                    "Expected bool, PathBuf, String, OsString, integer, or float",
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

            Ok(Self::Option(opt))
        }
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
