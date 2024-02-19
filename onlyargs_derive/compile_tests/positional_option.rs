#[derive(Debug, onlyargs_derive::OnlyArgs)]
struct Args {
    #[positional]
    rest: Option<i32>,
}

fn main() {}
