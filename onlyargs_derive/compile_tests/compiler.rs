#[test]
fn compile_tests() {
    let t = trybuild::TestCases::new();
    t.pass("compile_tests/default_bool_false.rs");
    t.pass("compile_tests/default_bool_true.rs");
    t.pass("compile_tests/default_f32.rs");
    t.pass("compile_tests/default_f64.rs");
    t.pass("compile_tests/default_i8.rs");
    t.pass("compile_tests/default_i128.rs");
    t.pass("compile_tests/default_isize.rs");
    // TODO: Negatives are not supported yet!
    // t.pass("compile_tests/default_negative_i8.rs");
    // t.pass("compile_tests/default_negative_i128.rs");
    // t.pass("compile_tests/default_negative_isize.rs");
    t.pass("compile_tests/default_osstring.rs");
    t.pass("compile_tests/default_pathbuf.rs");
    t.pass("compile_tests/default_string.rs");
    t.pass("compile_tests/default_u8.rs");
    t.pass("compile_tests/default_u128.rs");
    t.pass("compile_tests/default_usize.rs");

    t.pass("compile_tests/positional_f32.rs");
    t.pass("compile_tests/positional_f64.rs");
    t.pass("compile_tests/positional_i8.rs");
    t.pass("compile_tests/positional_i128.rs");
    t.pass("compile_tests/positional_isize.rs");
    t.pass("compile_tests/positional_osstring.rs");
    t.pass("compile_tests/positional_pathbuf.rs");
    t.pass("compile_tests/positional_string.rs");
    t.pass("compile_tests/positional_u8.rs");
    t.pass("compile_tests/positional_usize.rs");

    t.pass("compile_tests/empty.rs");
    t.pass("compile_tests/optional.rs");
    t.pass("compile_tests/struct_doc_comment.rs");
    t.pass("compile_tests/struct_footer.rs");

    t.compile_fail("compile_tests/conflicting_short_name.rs");
    t.pass("compile_tests/manual_short_name.rs");
    t.pass("compile_tests/ignore_short_name.rs");
}
