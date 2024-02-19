#[derive(Debug, onlyargs_derive::OnlyArgs)]
struct Args {
    min: i32,

    #[short('x')]
    max: i32,
}

fn main() {}
