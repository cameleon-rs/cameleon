#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/register.rs");
    t.compile_fail("tests/wrong_endianess.rs");
    t.compile_fail("tests/wrong_access_right.rs");
}
