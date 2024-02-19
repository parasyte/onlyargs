#[derive(Debug, onlyargs_derive::OnlyArgs)]
struct Args {
    #[default(123)]
    opt_num: Option<u64>,
}

fn main() {}
