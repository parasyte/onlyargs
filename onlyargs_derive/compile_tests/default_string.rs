#[derive(Debug, onlyargs_derive::OnlyArgs)]
struct Args {
    #[default("foo bar")]
    name: String,
}

fn main() {}
