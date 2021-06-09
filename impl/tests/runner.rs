/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/macros/register.rs");
    t.pass("tests/macros/memory.rs");
    t.pass("tests/macros/visibility.rs");
    t.pass("tests/macros/bitfield.rs");

    t.compile_fail("tests/macros/forbidden_visibility.rs");
    t.compile_fail("tests/macros/wrong_access_right.rs");
    t.compile_fail("tests/macros/wrong_endianness.rs");
    t.compile_fail("tests/macros/wrong_init_array.rs");
}
