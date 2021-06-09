/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[cfg(feature = "libusb")]
#[test]
fn test_examples() {
    let t = trybuild::TestCases::new();
    t.pass("examples/u3v/register_map.rs");
}
