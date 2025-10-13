![cameleon is a safe, fast, and flexible library for GenICam compatible cameras][logo]

[![Crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]
[![Build Status][actions-badge]][actions-url]
[![Actively Maintained](https://img.shields.io/badge/Maintenance%20Level-Actively%20Maintained-green.svg)](https://gist.github.com/cheerfulstoic/d107229326a01ff0f333a1d3476e068d)
[![MPL-2.0][mpl-badge]][mpl-url]

`cameleon` is a safe, fast, and flexible library for [GenICam][genicam-url] compatible cameras.

[logo]: https://raw.githubusercontent.com/cameleon-rs/cameleon/main/misc/logo.svg
[crates-badge]: https://img.shields.io/crates/v/cameleon.svg
[crates-url]: https://crates.io/crates/cameleon
[docs-badge]: https://docs.rs/cameleon/badge.svg
[docs-url]: https://docs.rs/cameleon
[mpl-badge]: https://img.shields.io/badge/License-MPL%202.0-brightgreen.svg
[mpl-url]: https://github.com/cameleon-rs/cameleon/blob/main/LICENSE
[actions-badge]: https://github.com/cameleon-rs/cameleon/workflows/CI/badge.svg
[actions-url]: https://github.com/cameleon-rs/cameleon/actions/workflows/ci.yml
[genicam-url]: https://www.emva.org/standards-technology/genicam/


## Overview

`cameleon` is a library for operating on `GenICam` compatible cameras.
Our main goal is to provide safe, fast, and flexible library for `GenICam` cameras.

Currently, `cameleon` supports only `USB3 Vision` cameras, but it's planned to support other protocols including `GigE Vision`. See [Roadmap][roadmap-url] for more details.

[roadmap-url]: https://github.com/cameleon-rs/cameleon#roadmap

## Usage

### USB3 Vision cameras
`cameleon` uses the pure-Rust [`nusb`][nusb-url] backend for USB3 Vision devices, so no external `libusb` installation is required. Enable the `usb` feature (the legacy `libusb` feature remains as an alias).

First, add dependencies like below.
```toml
[dependencies]
cameleon = { version = "0.1", features = ["usb"] }
```

Then, you can enumerate all cameras connected to the host, and start streaming.
```rust
use cameleon::u3v;

// Enumerates all cameras connected to the host.
let mut cameras = u3v::enumerate_cameras().unwrap();

if cameras.is_empty() {
    println!("no camera found");
    return;
}


let mut camera = cameras.pop().unwrap();

// Opens the camera.
camera.open().unwrap();
// Loads `GenApi` context. This is necessary for streaming.
camera.load_context().unwrap();

// Start streaming. Channel capacity is set to 3.
let payload_rx = camera.start_streaming(3).unwrap();

for _ in 0..10 {
    let payload = match payload_rx.recv_blocking() {
        Ok(payload) => payload,
        Err(e) => {
            println!("payload receive error: {e}");
            continue;
        }
    };
    println!(
        "payload received! block_id: {:?}, timestamp: {:?}",
        payload.id(),
        payload.timestamp()
    );
    if let Some(image_info) = payload.image_info() {
        println!("{:?}\n", image_info);
        let image = payload.image();
        // do something with the image.
        // ...
    }

    // Send back payload to streaming loop to reuse the buffer. This is optional.
    payload_rx.send_back(payload);
}

// Closes the camera.
camera.close().unwrap();
```

More examples can be found [here][cameleon-example].

[nusb-url]: https://docs.rs/nusb
[cameleon-example]: https://github.com/cameleon-rs/cameleon/tree/main/cameleon/examples


## Project Layout
`Cameleon` consists of several crates.

* [`cameleon`]: Provides high-level APIs to control cameras. This is the primary crate.
* [`cameleon-genapi`]: Provides parser and interpreter of `GenApi` XML.
* [`cameleon-device`]: Provides device specific protocol decoder and basic I/O operations for devices, also provides emulators.
* [`cameleon-gentl`]: Provides `GenTL` interfaces as a C library.
* [`cameleon-impl`]: Provides internal APIs for other crates. `cameleon-impl` is intended to be used only by `cameleon` project.
* [`cameleon-impl-macros`]: Provides procedural macros for other crates. `cameleon-impl-macros` is intended to be used only by `cameleon` project.

[`cameleon`]: https://github.com/cameleon-rs/cameleon/tree/main/cameleon
[`cameleon-genapi`]: https://github.com/cameleon-rs/cameleon/tree/main/genapi
[`cameleon-device`]: https://github.com/cameleon-rs/cameleon/tree/main/device
[`cameleon-gentl`]: https://github.com/cameleon-rs/cameleon/tree/main/gentl
[`cameleon-impl`]: https://github.com/cameleon-rs/cameleon/tree/main/impl
[`cameleon-impl-macros`]: https://github.com/cameleon-rs/cameleon/tree/main/impl/macros


## FAQ

### USB3 Vision

#### Platform notes
##### Linux/macOS
On Linux you may need to adjust device permissions so that your user can access USB cameras. You can add rules with `udev`; a configuration example is available in [`misc/u3v.rules`](misc/u3v.rules).

##### Windows
The backend relies on the WinUSB driver. You can install it for your device with [Zadig](https://zadig.akeo.ie/). When selecting your device in Zadig, ensure you choose the composite device entry, not the child interfaces, before installing WinUSB.  
![describe zadig list option][zadig-list-option]

![describe how to install WinUSB driver to your composite device][zadig-composite-device]
[zadig-list-option]: https://user-images.githubusercontent.com/6376004/123678264-11720d00-d881-11eb-98aa-eb649fdf3cb2.png
[zadig-composite-device]: https://user-images.githubusercontent.com/6376004/123937380-10e88c00-d9d1-11eb-9999-61439b6db788.png


#### Why is frame rate so low?
Frame rate can be affected by several reasons.

1. Parameter settings of the camera

`AcquisitionFrameRate` and `ExposureTime` directly affect frame rate. So you need to setup the parameters first to improve frame rate.
Also, if `DeviceLinkThroughputLimitMode` is set to `On`, you would need to increase the value of `DeviceLinkThroughputLimit`.

2. Many devices are streaming simultaneously on the same USB host controller

In this case, it's recommended to allocate the equal throughput limit to the connected cameras,
making sure that the total throughput does not exceed the maximum bandwidth of the host controller.

3. `usbfs_memory_mb` is set to low value

If you use Linux, you may need to increase `usbfs_memory_mb` limit.
By default, USB-FS on Linux systems only allows 16 MB of buffer memory for all USB devices. This is quite low for high-resolution image streaming.
We recommend you to set the value to 1000MB. You could set the value as following:
```sh
echo 1000 > /sys/module/usbcore/parameters/usbfs_memory_mb
```

## Roadmap
### [v0.2.0](https://github.com/cameleon-rs/cameleon/milestone/2)
* Add support for `GigE` cameras
* Impelment emulator
* Add support for saving and loading camera parameters

### [v0.3.0](https://github.com/cameleon-rs/cameleon/milestone/3)
* Implement payload chunk parser
* Add support for `GenTL`

### [v0.4.0](https://github.com/cameleon-rs/cameleon/milestone/4)
* Add support for `UVC` cameras

## Contributing
Thank you for your interest in contributing to `Cameleon`! We are so happy to have you join the development.  
To start developing, please refer to [CONTRIBUTING.md][contributing].

[contributing]: https://github.com/cameleon-rs/cameleon/blob/main/CONTRIBUTING.md

## Releasing
### 1. Publish
1. Check commits since the last release and determine whether they're semver-breaking.
2. Bump up all the crate versions and publish using [`cargo release <major|minor|patch>`](https://github.com/crate-ci/cargo-release)
3. Open a PR to reflect the changes.

### 2. Changelog
1. Create a new release on the GitHub page from the tag that `cargo release` has created
2. Use [automatically generated release notes](https://docs.github.com/en/repositories/releasing-projects-on-github/automatically-generated-release-notes) and modify it *manually*

## License
This project is licenced under [MPL 2.0][license].

[license]: https://github.com/cameleon-rs/cameleon/blob/main/LICENSE
