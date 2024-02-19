#[derive(Debug, onlyargs_derive::OnlyArgs)]
struct Args {
    #[positional]
    rest: Vec<f32>,
}

fn main() {}
