use proc_macro::{
    token_stream::IntoIter, Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream,
    TokenTree,
};

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

#[derive(Debug)]
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

#[derive(Debug)]
struct Attribute {
    name: Ident,
    stream: TokenStream,
}

struct Fork {
    iter: IntoIter,
    steps: usize,
}

impl ArgumentStruct {
    pub(crate) fn parse(mut input: IntoIter) -> Result<Self, TokenStream> {
        let attrs = parse_attributes(&mut input)?;
        parse_visibility(&mut input)?;
        expect_ident(&mut input, "struct")?;

        let name = parse_ident(&mut input)?;
        let content = parse_group(&mut input, Delimiter::Brace)?.into_iter();
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
            tree => Err(spanned_error("Unexpected token", parse_span(tree))),
        }
    }
}

impl Argument {
    fn parse(mut input: IntoIter) -> Result<Vec<Self>, TokenStream> {
        let mut args = vec![];

        // TODO: This loop looks awful!
        while input.clone().next().is_some() {
            let attrs = parse_attributes(&mut input)?;

            // Parse attributes
            let mut default = None;
            let mut long = false;
            let mut short = None;
            for attr in &attrs {
                let name = attr.name.to_string();
                match name.as_str() {
                    "default" => {
                        let mut stream = parse_group(
                            &mut attr.stream.clone().into_iter(),
                            Delimiter::Parenthesis,
                        )?
                        .into_iter();

                        default = Some(parse_literal(&mut stream)?);
                    }
                    "long" => long = true,
                    "short" => {
                        let mut stream = parse_group(
                            &mut attr.stream.clone().into_iter(),
                            Delimiter::Parenthesis,
                        )?
                        .into_iter();
                        let lit = parse_literal(&mut stream)?;

                        short = Some(parse_char_literal(lit)?);
                    }
                    _ => (),
                }
            }

            parse_visibility(&mut input)?;

            let mut fork = Fork::new(&input);
            if let Ok(mut flag) = ArgFlag::parse(&mut fork) {
                input.by_ref().take(fork.steps).for_each(|_| ());

                // Patch flag with attributes
                flag.doc = get_doc_comment(&attrs);

                if long {
                    flag.short = None;
                } else if short.is_some() {
                    flag.short = short;
                }

                args.push(Self::Flag(flag))
            } else {
                match ArgOption::parse(&mut input) {
                    Ok(mut opt) => {
                        opt.doc = get_doc_comment(&attrs);

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

                        if long {
                            opt.short = None;
                        } else if short.is_some() {
                            opt.short = short;
                        }

                        args.push(Self::Option(opt));
                    }
                    Err(err) => return Err(err),
                }
            }
        }

        Ok(args)
    }
}

impl ArgFlag {
    fn parse(input: &mut Fork) -> Result<Self, TokenStream> {
        let name = parse_ident(&mut input.iter)?;
        input.steps += 1;
        expect_punct(&mut input.iter, ':')?;
        input.steps += 1;

        expect_ident(&mut input.iter, "bool")?;
        input.steps += 1;

        if expect_punct(&mut input.iter, ',').is_ok() {
            input.steps += 1;
        }

        // TODO: Add an attribute to disable short names
        let short = name.to_string().chars().find(|ch| ch.is_ascii_alphabetic());

        Ok(ArgFlag {
            name,
            short,
            doc: vec![],
            output: true,
        })
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

impl ArgOption {
    fn parse(input: &mut IntoIter) -> Result<Self, TokenStream> {
        let name = parse_ident(input)?;
        expect_punct(input, ':')?;

        let (path, span) = parse_path(input)?;
        let path = path.as_str();

        // We have to check multiple possible paths for types that are not included in
        // `std::prelude`. The type system is not available here, so we need to make some educated
        // guesses about field types.
        let required_paths = [
            "::std::path::PathBuf",
            "std::path::PathBuf",
            "path::PathBuf",
            "PathBuf",
        ];
        let required_os_strings = [
            "::std::ffi::OsString",
            "std::ffi::OsString",
            "ffi::OsString",
            "OsString",
        ];
        let required_numbers = [
            "f32", "f64", "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64",
            "u128", "usize",
        ];
        let positional_paths = [
            "Vec<::std::path::PathBuf>",
            "Vec<std::path::PathBuf>",
            "Vec<path::PathBuf>",
            "Vec<PathBuf>",
        ];
        let positional_os_strings = [
            "Vec<::std::ffi::OsString>",
            "Vec<std::ffi::OsString>",
            "Vec<ffi::OsString>",
            "Vec<OsString>",
        ];
        let positional_numbers = [
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
        let optional_paths = [
            "Option<::std::path::PathBuf>",
            "Option<std::path::PathBuf>",
            "Option<path::PathBuf>",
            "Option<PathBuf>",
        ];
        let optional_os_strings = [
            "Option<::std::ffi::OsString>",
            "Option<std::ffi::OsString>",
            "Option<ffi::OsString>",
            "Option<OsString>",
        ];
        let optional_numbers = [
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

        let optional = if optional_paths.contains(&path)
            || optional_os_strings.contains(&path)
            || optional_numbers.contains(&path)
            || path == "Option<String>"
            || positional_paths.contains(&path)
            || positional_os_strings.contains(&path)
            || positional_numbers.contains(&path)
            || path == "Vec<String>"
        {
            true
        } else if required_paths.contains(&path)
            || required_os_strings.contains(&path)
            || required_numbers.contains(&path)
            || path == "String"
        {
            false
        } else {
            return Err(spanned_error(
                "Expected bool, PathBuf, String, OsString, integer or float",
                span,
            ));
        };

        let ty_help = if optional_paths.contains(&path)
            || required_paths.contains(&path)
            || positional_paths.contains(&path)
        {
            ArgType::Path
        } else if optional_os_strings.contains(&path)
            || required_os_strings.contains(&path)
            || positional_os_strings.contains(&path)
        {
            ArgType::OsString
        } else if path == "String" || path == "Vec<String>" || path == "Option<String>" {
            ArgType::String
        } else if optional_numbers.contains(&path)
            || required_numbers.contains(&path)
            || positional_numbers.contains(&path)
        {
            ArgType::Number
        } else {
            unreachable!();
        };

        let positional = positional_paths.contains(&path)
            || positional_os_strings.contains(&path)
            || positional_numbers.contains(&path)
            || path == "Vec<String>";

        // TODO: Add an attribute to disable short names
        let short = name.to_string().chars().find(|ch| ch.is_ascii_alphabetic());

        Ok(ArgOption {
            name,
            short,
            ty_help,
            doc: vec![],
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

impl Fork {
    fn new(iter: &IntoIter) -> Self {
        Self {
            iter: iter.clone(),
            steps: 0,
        }
    }
}

pub(crate) fn spanned_error<S: AsRef<str>>(msg: S, span: Span) -> TokenStream {
    let mut group = Group::new(
        Delimiter::Parenthesis,
        TokenTree::from(Literal::string(msg.as_ref())).into(),
    );
    group.set_span(span);

    TokenStream::from_iter([
        TokenTree::Ident(Ident::new("compile_error", span)),
        TokenTree::Punct(Punct::new('!', Spacing::Alone)),
        TokenTree::Group(group),
        TokenTree::Punct(Punct::new(';', Spacing::Alone)),
    ])
}

fn get_doc_comment(attrs: &[Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter_map(|attr| {
            if attr.name.to_string() == "doc" {
                let mut stream = attr.stream.clone().into_iter();

                match stream.next() {
                    Some(TokenTree::Punct(punct)) if punct.as_char() == '=' => (),
                    _ => return None,
                }

                parse_literal(&mut stream)
                    .and_then(parse_string_literal)
                    .ok()
            } else {
                None
            }
        })
        .collect()
}

fn parse_attributes(input: &mut IntoIter) -> Result<Vec<Attribute>, TokenStream> {
    let mut attrs = vec![];

    loop {
        match input.clone().next() {
            Some(TokenTree::Punct(punct)) if punct.as_char() == '#' => input.next(),
            _ => break,
        };

        let mut group = match input.next() {
            Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Bracket => {
                group.stream()
            }
            tree => return Err(spanned_error("Expected `[`", parse_span(tree))),
        }
        .into_iter();

        match group.next() {
            Some(TokenTree::Ident(ident)) => {
                attrs.push(Attribute {
                    name: ident,
                    stream: TokenStream::from_iter(group),
                });
            }
            tree => return Err(spanned_error("Expected ident", parse_span(tree))),
        }
    }

    Ok(attrs)
}

fn parse_visibility(input: &mut IntoIter) -> Result<(), TokenStream> {
    match input.clone().next() {
        Some(TokenTree::Ident(ident)) if ident.to_string() == "pub" => input.next(),
        _ => return Ok(()),
    };

    match input.clone().next() {
        Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis => {
            input.next();
        }
        _ => return Ok(()),
    }

    Ok(())
}

fn parse_path(input: &mut IntoIter) -> Result<(String, Span), TokenStream> {
    let mut path = String::new();
    let mut span = None;

    for tree in input.by_ref() {
        match tree {
            TokenTree::Punct(punct) if punct.as_char() == ',' => break,
            TokenTree::Punct(punct) => {
                span.get_or_insert_with(|| punct.span());
                path.push(punct.as_char());
            }
            TokenTree::Ident(ident) => {
                span.get_or_insert_with(|| ident.span());
                path.push_str(&ident.to_string());
            }
            tree => return Err(spanned_error("Unexpected token", parse_span(Some(tree)))),
        }
    }

    let span = span.ok_or_else(|| spanned_error("Unexpected end of stream", Span::call_site()))?;

    Ok((path, span))
}

fn expect_ident(input: &mut IntoIter, expect: &str) -> Result<(), TokenStream> {
    parse_ident(input).and_then(|ident| {
        if ident.to_string() == expect {
            Ok(())
        } else {
            Err(spanned_error(format!("Expected `{expect}`"), ident.span()))
        }
    })
}

fn expect_punct(input: &mut IntoIter, expect: char) -> Result<(), TokenStream> {
    parse_punct(input).and_then(|punct| {
        if punct.as_char() == expect {
            Ok(())
        } else {
            Err(spanned_error(format!("Expected `{expect}`"), punct.span()))
        }
    })
}

fn parse_group(input: &mut IntoIter, delim: Delimiter) -> Result<TokenStream, TokenStream> {
    Ok(match input.next() {
        Some(TokenTree::Group(group)) if group.delimiter() == delim => group.stream(),
        tree => {
            let delim = match delim {
                Delimiter::Brace => "{",
                Delimiter::Bracket => "[",
                Delimiter::None => "delimiter",
                Delimiter::Parenthesis => "(",
            };
            return Err(spanned_error(format!("Expected {delim}"), parse_span(tree)));
        }
    })
}

fn parse_ident(input: &mut IntoIter) -> Result<Ident, TokenStream> {
    Ok(match input.next() {
        Some(TokenTree::Ident(ident)) => ident,
        tree => return Err(spanned_error("Expected identifier", parse_span(tree))),
    })
}

fn parse_literal(input: &mut IntoIter) -> Result<Literal, TokenStream> {
    Ok(match input.next() {
        Some(TokenTree::Literal(lit)) => lit,
        tree => return Err(spanned_error("Expected literal", parse_span(tree))),
    })
}

fn parse_punct(input: &mut IntoIter) -> Result<Punct, TokenStream> {
    Ok(match input.next() {
        Some(TokenTree::Punct(punct)) => punct,
        tree => return Err(spanned_error("Expected punctuation", parse_span(tree))),
    })
}

fn parse_char_literal(lit: Literal) -> Result<char, TokenStream> {
    let string = format!("{lit}");
    if string.len() == 3 || !string.starts_with('\'') || !string.ends_with('\'') {
        Err(spanned_error("Expected char literal", lit.span()))
    } else {
        // Strip single quotes.
        string
            .chars()
            .nth(1)
            .ok_or_else(|| spanned_error("Expected char literal", lit.span()))
    }
}

fn parse_string_literal(lit: Literal) -> Result<String, TokenStream> {
    let string = format!("{lit}");
    if !string.starts_with('"') || !string.ends_with('"') {
        Err(spanned_error("Expected string literal", lit.span()))
    } else {
        // Strip double quotes and escapes.
        Ok(string[1..string.len() - 1]
            .trim()
            .replace(r#"\""#, r#"""#)
            .replace(r"\n", "\n")
            .replace(r"\r", "\r")
            .replace(r"\t", "\t")
            .replace(r"\'", "'")
            .replace(r"\\", r"\"))
    }
}

fn parse_span(opt: Option<TokenTree>) -> Span {
    match opt {
        Some(TokenTree::Group(group)) => group.span(),
        Some(TokenTree::Ident(ident)) => ident.span(),
        Some(TokenTree::Punct(punct)) => punct.span(),
        Some(TokenTree::Literal(lit)) => lit.span(),
        None => Span::call_site(),
    }
}
