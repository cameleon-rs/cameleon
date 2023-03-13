/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This example describes how to start streaming and receive payloads.

use cameleon::u3v::enumerate_cameras;

fn main() {
    // Enumerates cameras connected to the host.
    let mut cameras = enumerate_cameras().unwrap();

    if cameras.is_empty() {
        println!("no camera found!");
        return;
    }

    let mut camera = cameras.pop().unwrap();

    // Open the camera.
    camera.open().unwrap();
    // Load `GenApi` context.
    camera.load_context().unwrap();

    // Start streaming. Channel capacity is set to 3.
    let payload_rx = camera.start_streaming(3).unwrap();

    for _ in 0..10 {
        let payload = payload_rx
            .recv_blocking()
            .expect("should receive a payload");
        println!(
            "payload received! block_id: {:?}, timestamp: {:?}",
            payload.id(),
            payload.timestamp()
        );
        if let Some(image_info) = payload.image_info() {
            println!("{:?}\n", image_info);
        }

        // Send back payload to streaming loop to reuse the buffer.
        payload_rx.send_back(payload);
    }

    camera.close().ok();
}
