#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/macros/register.rs");
    t.pass("tests/macros/memory.rs");
    t.pass("tests/macros/visibility.rs");

    t.compile_fail("tests/macros/forbidden_visibility.rs");
    t.compile_fail("tests/macros/wrong_access_right.rs");
    t.compile_fail("tests/macros/wrong_endianness.rs");
    t.compile_fail("tests/macros/wrong_ty_attr.rs");
    t.compile_fail("tests/macros/wrong_init_array.rs");
}
