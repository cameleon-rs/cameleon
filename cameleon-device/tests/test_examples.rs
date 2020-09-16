#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("examples/usb3/device_control.rs");
    t.pass("examples/usb3/device_enumeration.rs");
}
