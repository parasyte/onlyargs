#[derive(Debug, onlyargs_derive::OnlyArgs)]
struct Args {
    #[default("./foo/bar")]
    path: std::path::PathBuf,
}

fn main() {}
