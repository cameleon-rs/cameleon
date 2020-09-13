#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/register.rs");
    t.compile_fail("tests/wrong_endianess.rs");
    t.compile_fail("tests/wrong_access_right.rs");
    t.compile_fail("tests/wrong_ty_attr1.rs");
    t.compile_fail("tests/wrong_ty_attr2.rs");

    t.pass("tests/memory.rs");
}
