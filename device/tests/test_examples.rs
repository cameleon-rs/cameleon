#[cfg(feature = "libusb")]
#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("examples/u3v/device_control.rs");
    t.pass("examples/u3v/device_enumeration.rs");
}
