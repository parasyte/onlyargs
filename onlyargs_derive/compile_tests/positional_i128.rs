#[derive(Debug, onlyargs_derive::OnlyArgs)]
struct Args {
    #[positional]
    rest: Vec<i128>,
}

fn main() {}
