#[test]
fn test_examples() {
    let t = trybuild::TestCases::new();
    t.pass("examples/device/u3v/register_map.rs");
}
