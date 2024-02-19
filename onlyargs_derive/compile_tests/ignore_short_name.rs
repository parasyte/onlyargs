#[derive(Debug, onlyargs_derive::OnlyArgs)]
struct Args {
    min: i32,

    #[long]
    max: i32,
}

fn main() {}
