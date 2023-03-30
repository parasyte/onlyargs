use syn::parse::{discouraged::Speculative as _, Parse, ParseStream};
use syn::{braced, parse_quote, Attribute, Expr, ExprLit, Ident, Lit, Path, Token, Visibility};

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
    pub(crate) optional: bool,
    pub(crate) positional: bool,
}

#[derive(Debug)]
pub(crate) struct ArgView<'a> {
    pub(crate) name: &'a Ident,
    pub(crate) short: Option<char>,
    pub(crate) ty_help: ArgType,
    pub(crate) doc: &'a [String],
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum ArgType {
    Bool,
    Number,
    OsString,
    Path,
    String,
}

impl Parse for ArgumentStruct {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = Attribute::parse_outer(input)?;
        input.parse::<Visibility>()?;
        input.parse::<Token![struct]>()?;

        let name = input.parse()?;

        let content;
        braced!(content in input);
        let fields = content
            .parse_terminated(Argument::parse, Token![,])?
            .into_pairs()
            .map(|pair| pair.into_value())
            .collect::<Vec<_>>();

        let mut flags = vec![];
        let mut options = vec![];
        let mut positional = None;

        for field in fields {
            match field {
                Argument::Flag(flag) => flags.push(flag),
                Argument::Option(opt) => match (opt.positional, &positional) {
                    (true, None) => positional = Some(opt),
                    (true, Some(_)) => {
                        return Err(input.error("Positional arguments can only be specified once."));
                    }
                    _ => options.push(opt),
                },
            }
        }

        let doc = get_doc_comment(&attrs).unwrap_or_default();

        Ok(Self {
            name,
            flags,
            options,
            positional,
            doc,
        })
    }
}

impl Parse for Argument {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = Attribute::parse_outer(input)?;
        input.parse::<Visibility>()?;

        let fork = input.fork();
        if let Ok(mut flag) = fork.parse::<ArgFlag>() {
            input.advance_to(&fork);
            flag.doc = get_doc_comment(&attrs).unwrap_or_default();

            Ok(Self::Flag(flag))
        } else if let Ok(mut opt) = input.parse::<ArgOption>() {
            opt.doc = get_doc_comment(&attrs).unwrap_or_default();

            Ok(Self::Option(opt))
        } else {
            Err(input.error("Expected a type suitable for CLI arguments"))
        }
    }
}

impl Parse for ArgFlag {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;

        // Any of the supported types
        let ty: Path = input.parse()?;

        if ty == parse_quote!(bool) {
            // TODO: Add an attribute to disable short names
            let short = name.to_string().chars().find(|ch| ch.is_ascii_alphabetic());

            Ok(ArgFlag {
                name,
                short,
                doc: vec![],
                output: true,
            })
        } else {
            Err(input.error("Expected a bool"))
        }
    }
}

impl Parse for ArgOption {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;

        // Any of the supported types
        let ty: Path = input.parse()?;

        // We have to check multiple possible paths for types that are not included in
        // `std::prelude`. The type system is not available here, so we need to make some educated
        // guesses about field types.
        let required_paths = [
            parse_quote!(::std::path::PathBuf),
            parse_quote!(std::path::PathBuf),
            parse_quote!(path::PathBuf),
            parse_quote!(PathBuf),
        ];
        let required_os_strings = [
            parse_quote!(::std::ffi::OsString),
            parse_quote!(std::ffi::OsString),
            parse_quote!(ffi::OsString),
            parse_quote!(OsString),
        ];
        let required_numbers = [
            parse_quote!(f32),
            parse_quote!(f64),
            parse_quote!(i8),
            parse_quote!(i16),
            parse_quote!(i32),
            parse_quote!(i64),
            parse_quote!(i128),
            parse_quote!(isize),
            parse_quote!(u8),
            parse_quote!(u16),
            parse_quote!(u32),
            parse_quote!(u64),
            parse_quote!(u128),
            parse_quote!(usize),
        ];
        let positional_paths = [
            parse_quote!(Vec<::std::path::PathBuf>),
            parse_quote!(Vec<std::path::PathBuf>),
            parse_quote!(Vec<path::PathBuf>),
            parse_quote!(Vec<PathBuf>),
        ];
        let positional_os_strings = [
            parse_quote!(Vec<::std::ffi::OsString>),
            parse_quote!(Vec<std::ffi::OsString>),
            parse_quote!(Vec<ffi::OsString>),
            parse_quote!(Vec<OsString>),
        ];
        let positional_numbers = [
            parse_quote!(Vec<f32>),
            parse_quote!(Vec<f64>),
            parse_quote!(Vec<i8>),
            parse_quote!(Vec<i16>),
            parse_quote!(Vec<i32>),
            parse_quote!(Vec<i64>),
            parse_quote!(Vec<i128>),
            parse_quote!(Vec<isize>),
            parse_quote!(Vec<u8>),
            parse_quote!(Vec<u16>),
            parse_quote!(Vec<u32>),
            parse_quote!(Vec<u64>),
            parse_quote!(Vec<u128>),
            parse_quote!(Vec<usize>),
        ];
        let optional_paths = [
            parse_quote!(Option<::std::path::PathBuf>),
            parse_quote!(Option<std::path::PathBuf>),
            parse_quote!(Option<path::PathBuf>),
            parse_quote!(Option<PathBuf>),
        ];
        let optional_os_strings = [
            parse_quote!(Option<::std::ffi::OsString>),
            parse_quote!(Option<std::ffi::OsString>),
            parse_quote!(Option<ffi::OsString>),
            parse_quote!(Option<OsString>),
        ];
        let optional_numbers = [
            parse_quote!(Option<f32>),
            parse_quote!(Option<f64>),
            parse_quote!(Option<i8>),
            parse_quote!(Option<i16>),
            parse_quote!(Option<i32>),
            parse_quote!(Option<i64>),
            parse_quote!(Option<i128>),
            parse_quote!(Option<isize>),
            parse_quote!(Option<u8>),
            parse_quote!(Option<u16>),
            parse_quote!(Option<u32>),
            parse_quote!(Option<u64>),
            parse_quote!(Option<u128>),
            parse_quote!(Option<usize>),
        ];

        let optional = if optional_paths.contains(&ty)
            || optional_os_strings.contains(&ty)
            || optional_numbers.contains(&ty)
            || ty == parse_quote!(Option<String>)
        {
            true
        } else if required_paths.contains(&ty)
            || required_os_strings.contains(&ty)
            || required_numbers.contains(&ty)
            || ty == parse_quote!(String)
            || positional_paths.contains(&ty)
            || positional_os_strings.contains(&ty)
            || positional_numbers.contains(&ty)
            || ty == parse_quote!(Vec<String>)
        {
            false
        } else {
            return Err(input.error("Expected bool, PathBuf, String, OsString, integer or float"));
        };

        let ty_help = if optional_paths.contains(&ty)
            || required_paths.contains(&ty)
            || positional_paths.contains(&ty)
        {
            ArgType::Path
        } else if optional_os_strings.contains(&ty)
            || required_os_strings.contains(&ty)
            || positional_os_strings.contains(&ty)
        {
            ArgType::OsString
        } else if ty == parse_quote!(String)
            || ty == parse_quote!(Vec<String>)
            || ty == parse_quote!(Option<String>)
        {
            ArgType::String
        } else if optional_numbers.contains(&ty)
            || required_numbers.contains(&ty)
            || positional_numbers.contains(&ty)
        {
            ArgType::Number
        } else {
            unreachable!();
        };

        let positional = positional_paths.contains(&ty)
            || positional_os_strings.contains(&ty)
            || positional_numbers.contains(&ty)
            || ty == parse_quote!(Vec<String>);

        // TODO: Add an attribute to disable short names
        let short = name.to_string().chars().find(|ch| ch.is_ascii_alphabetic());

        Ok(ArgOption {
            name,
            short,
            ty_help,
            doc: vec![],
            optional,
            positional,
        })
    }
}

impl ArgFlag {
    pub(crate) fn as_view(&self) -> ArgView {
        ArgView {
            name: &self.name,
            short: self.short,
            ty_help: ArgType::Bool,
            doc: &self.doc,
        }
    }
}

impl ArgOption {
    pub(crate) fn as_view(&self) -> ArgView {
        ArgView {
            name: &self.name,
            short: self.short,
            ty_help: self.ty_help,
            doc: &self.doc,
        }
    }
}

impl ArgType {
    pub(crate) fn as_str(&self) -> &str {
        match self {
            Self::Bool => "",
            Self::Number => " NUMBER",
            Self::OsString | Self::String => " STRING",
            Self::Path => " PATH",
        }
    }
}

fn get_doc_comment(attrs: &[Attribute]) -> Option<Vec<String>> {
    let doc = attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("doc") {
                attr.meta
                    .require_name_value()
                    .map(|nv| match &nv.value {
                        Expr::Lit(ExprLit {
                            lit: Lit::Str(lit), ..
                        }) => Some(lit.value().trim().to_string()),
                        _ => None,
                    })
                    .ok()
                    .flatten()
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    if !doc.is_empty() {
        Some(doc)
    } else {
        None
    }
}
