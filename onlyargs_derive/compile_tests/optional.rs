#[derive(Debug, onlyargs_derive::OnlyArgs)]
struct Args {
    #[long]
    opt_f32: Option<f32>,
    #[long]
    opt_f64: Option<f64>,
    #[long]
    opt_i8: Option<i8>,
    #[long]
    opt_i16: Option<i16>,
    #[long]
    opt_i32: Option<i32>,
    #[long]
    opt_i64: Option<i64>,
    #[long]
    opt_i128: Option<i128>,
    #[long]
    opt_isize: Option<isize>,
    #[long]
    opt_u8: Option<u8>,
    #[long]
    opt_u16: Option<u16>,
    #[long]
    opt_u32: Option<u32>,
    #[long]
    opt_u64: Option<u64>,
    #[long]
    opt_u128: Option<u128>,
    #[long]
    opt_usize: Option<usize>,
    #[long]
    opt_osstring: Option<std::ffi::OsString>,
    #[long]
    opt_path: Option<std::path::PathBuf>,
    #[long]
    opt_string: Option<String>,
}

fn main() {}
