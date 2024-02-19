#[derive(Debug, onlyargs_derive::OnlyArgs)]
struct Args {
    #[default("foo bar")]
    name: std::ffi::OsString,
}

fn main() {}
