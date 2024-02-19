#[derive(Debug, onlyargs_derive::OnlyArgs)]
struct Args {
    #[positional]
    #[default(123)]
    nums: Vec<u64>,
}

fn main() {}
