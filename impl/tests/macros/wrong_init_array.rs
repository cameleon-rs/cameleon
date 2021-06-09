/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use cameleon_impl::memory::register_map;

#[register_map(base = 0, endianness = LE)]
enum ABRM {
    #[register(len = 4, access = RO, ty = Bytes)]
    ProtocolEndianness = &[0xFF, 1000, 0xFF, 0xFF],
}

fn main() {}
