#[derive(Debug, onlyargs_derive::OnlyArgs)]
struct Args {
    #[positional]
    rest: Vec<String>,
    #[positional]
    more: Vec<String>,
}

fn main() {}
