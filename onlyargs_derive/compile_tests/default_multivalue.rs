#[derive(Debug, onlyargs_derive::OnlyArgs)]
struct Args {
    #[default(123)]
    nums: Vec<u64>,
}

fn main() {}
