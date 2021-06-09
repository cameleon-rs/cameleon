/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

extern crate cameleon_device;

use cameleon_device::u3v::enumerate_devices;

fn main() {
    let devices = enumerate_devices().unwrap();
    if devices.is_empty() {
        println!("no device found");
    }

    for device in devices {
        println! {"{}", device.device_info};
    }
}
