<p align="center">
  <img src="../misc/logo.svg">
</p>

`cameleon` is a safe, fast, and flexible library for [GenICam](https://www.emva.org/standards-technology/genicam/) compatible cameras.

## Overview

`cameleon` is a library for operating on `GenICam` compatible cameras.  
Our main goal is to provide safe, fast, and flexible library for `GenICam` cameras.
Currently, `cameleon` supports only `USB3 Vision` cameras, but it's planned to support other protocols including `GigE Vision`. See [Roadmap](#Roadmap) for more details.


## Usage

### USB3 Vision cameras
First, you need to install `libusb` to communicate with `U3V` cameras, and add the following to your `Cargo.toml`:

```toml
[dependencies]
cameleon = { version = 0.1, features = 'libusb'}
```

Then you can enumerate all cameras connected to the host, and start streaming.

```rust
use cameleon::u3v;

// Enumerates all cameras connected to the host.
let cameras = u3v::enumerate_cameras().unwrap();

if cameras.is_empty() {
    println!("no camera found")
    return;
}


let mut camera = cameras.pop().unwrap();

// Opens the camera.
camera.open().unwrap();
// Loads `GenApi` context. This is necessary for streaming.
camera.load_context().unwrap();

// Start streaming.
let payload_rx = camera.start_streaming(10).unwrap();

let mut payload_count = 0;
while payload_count < 10 {
    match payload_rx.try_recv() {
        Ok(payload) => {
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
            payload_count += 1;

            // Send back payload to streaming loop to reuse the buffer. This is optional.
            payload_rx.send_back(payload);
        }
        Err(_err) => {
            continue;
        }
    }
}

// Closes the camera.
camera.close().unwrap();
```

More examples can be found [here](examples).

## FAQ

### USB3 Vision

#### Why isn't a camera found even though it is connected to the host?
It's probably due to permission issue for USB devices. You could add permissions by editing `udev` rules, a configuration example is found [here](../misc/u3v.rules).

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

### [v0.3.0](https://github.com/cameleon-rs/cameleon/milestone/3)
* Implement payload chunk parser
* Add support for `GenTL`

### [v0.4.0](https://github.com/cameleon-rs/cameleon/milestone/4)
* Add support for `UVC` cameras

## Release cycle
We continuously update the minor version every four weeks, until the version reaches `1.0.0`.

## Contributing
Thank you for your interest in contributing to `Cameleon`! We are so happy to have you join the development.  
To start developing, please refer to [CONTRIBUTING.md](../CONTRIBUTING.md).

## License
This project is licenced under [MPL 2.0](../LICENSE).
