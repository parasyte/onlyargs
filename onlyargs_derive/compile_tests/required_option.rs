#[derive(Debug, onlyargs_derive::OnlyArgs)]
struct Args {
    #[required]
    required_option: Option<String>,
}

fn main() {}
