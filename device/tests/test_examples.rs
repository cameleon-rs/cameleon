/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[cfg(feature = "usb")]
#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("examples/u3v/device_control.rs");
    t.pass("examples/u3v/device_enumeration.rs");
}
