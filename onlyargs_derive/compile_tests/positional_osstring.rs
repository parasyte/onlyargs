#[derive(Debug, onlyargs_derive::OnlyArgs)]
struct Args {
    #[positional]
    rest: Vec<std::ffi::OsString>,
}

fn main() {}
