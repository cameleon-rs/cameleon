#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/macros/register.rs");
    t.pass("tests/macros/memory.rs");

    t.compile_fail("tests/macros/wrong_endianess.rs");
    t.compile_fail("tests/macros/wrong_access_right.rs");
    t.compile_fail("tests/macros/wrong_ty_attr1.rs");
    t.compile_fail("tests/macros/wrong_ty_attr2.rs");
}
