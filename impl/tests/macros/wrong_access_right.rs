/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use cameleon_impl::memory::register_map;

#[register_map(base = 0, endianness = LE)]
pub enum ABRM {
    #[register(len = 2, access = Ro, ty = u16)]
    GenCpVersionMinor = 321,

    #[register(len = 8, access = RO, ty = u64)]
    SBRMAddress = SBRM_ADDRESS,
}

fn main() {}
