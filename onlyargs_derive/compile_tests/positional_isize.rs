#[derive(Debug, onlyargs_derive::OnlyArgs)]
struct Args {
    #[positional]
    rest: Vec<isize>,
}

fn main() {}
