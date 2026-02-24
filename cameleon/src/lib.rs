/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! `cameleon` is a safe, fast, and flexible library for [GenICam][genicam-url] compatible cameras.
//!
//! [genicam-url]: https://www.emva.org/standards-technology/genicam/
//!
//! ## Overview
//!
//! `cameleon` is a library for operating on `GenICam` compatible cameras.
//! Our main goal is to provide safe, fast, and flexible library for `GenICam` cameras.
//!
//! Currently, `cameleon` supports only `USB3 Vision` cameras, but it's planned to support other protocols including `GigE Vision`. See [Roadmap][roadmap-url] for more details.
//!
//! [roadmap-url]: https://github.com/cameleon-rs/cameleon#roadmap
//!
//! ## Usage
//!
//! ### USB3 Vision cameras
//! You need to install `libusb` to use USB3 Vision cameras, see [How to install
//! `libusb`](#how-to-install-libusb).
//!
//! First, add dependencies like below.
//! ```toml
//! [dependencies]
//! cameleon = { version = "0.1", features = ["libusb"] }
//! ```
//!
//! Then, you can enumerate all cameras connected to the host, and start streaming.
//! ```rust
//! use cameleon::u3v;
//!
//! // Enumerates all cameras connected to the host.
//! let mut cameras = u3v::enumerate_cameras().unwrap();
//!
//! if cameras.is_empty() {
//!     println!("no camera found");
//!     return;
//! }
//!
//!
//! let mut camera = cameras.pop().unwrap();
//!
//! // Opens the camera.
//! camera.open().unwrap();
//! // Loads `GenApi` context. This is necessary for streaming.
//! camera.load_context().unwrap();
//!
//! // Start streaming. Channel capacity is set to 3.
//! let payload_rx = camera.start_streaming(3).unwrap();
//!
//! let mut payload_count = 0;
//! while payload_count < 10 {
//!     match payload_rx.try_recv() {
//!         Ok(payload) => {
//!             println!(
//!                 "payload received! block_id: {:?}, timestamp: {:?}",
//!                 payload.id(),
//!                 payload.timestamp()
//!             );
//!             if let Some(image_info) = payload.image_info() {
//!                 println!("{:?}\n", image_info);
//!                 let image = payload.image();
//!                 // do something with the image.
//!                 // ...
//!             }
//!             payload_count += 1;
//!
//!             // Send back payload to streaming loop to reuse the buffer. This is optional.
//!             payload_rx.send_back(payload);
//!         }
//!         Err(_err) => {
//!             continue;
//!         }
//!     }
//! }
//!
//! // Closes the camera.
//! camera.close().unwrap();
//! ```
//!
//! More examples can be found [here][cameleon-example].
//!
//! [libusb-url]: https://libusb.info
//! [cameleon-example]: https://github.com/cameleon-rs/cameleon/tree/main/cameleon/examples
//!
//! ## FAQ
//! ### USB3 Vision
//! #### How to install `libusb`
//! ##### Linux/macOS
//! You need to install [libusb][libusb-url] to the place where `pkg-config` can find. Basically all you have to do is just installing `libusb` through your system package manager like `sudo apt install libusb-1.0-0-dev` or `brew install libusb`.
//!
//! If you use Linux, it's probably needed to edit permissions for USB devices. You could add permissions by editing `udev` rules, a configuration example is found [here](https://github.com/cameleon-rs/cameleon/blob/main/misc/u3v.rules).
//!
//! ##### Windows
//! You need to install [libusb][libusb-url] with `vcpkg`, please see [here][libusb-vcpkg] to install `libusb` with `vcpkg`.
//!
//! Also, you need to install a driver for your device. You can find resource [here][libusb-driver-installation] for driver-installation.  
//! NOTE: Make sure to install a driver to a composite device not to its child devices.  
//! To do this, you need to list all devices connected to the host computer with `zadig` like below.  
//! ![describe zadig list option][zadig-list-option]
//!
//! Then install `WinUSB` to your device.  
//! ![describe how to install WinUSB driver to your composite device][zadig-composite-device]
//!
//! [libusb-vcpkg]: https://github.com/libusb/libusb/wiki/Windows#vcpkg_port
//! [libusb-driver-installation]: https://github.com/libusb/libusb/wiki/Windows#driver-installation
//! [zadig-list-option]: https://user-images.githubusercontent.com/6376004/123678264-11720d00-d881-11eb-98aa-eb649fdf3cb2.png
//! [zadig-composite-device]: https://user-images.githubusercontent.com/6376004/123937380-10e88c00-d9d1-11eb-9999-61439b6db788.png
//!
//! #### Why is frame rate so low?
//! Frame rate can be affected by several reasons.
//!
//! 1. Parameter settings of the camera
//!
//! `AcquisitionFrameRate` and `ExposureTime` directly affect frame rate. So you need to setup the parameters first to improve frame rate.
//! Also, if `DeviceLinkThroughputLimitMode` is set to `On`, you would need to increase the value of `DeviceLinkThroughputLimit`.
//!
//! 2. Many devices are streaming simultaneously on the same USB host controller
//!
//! In this case, it's recommended to allocate the equal throughput limit to the connected cameras,
//! making sure that the total throughput does not exceed the maximum bandwidth of the host controller.
//!
//! 3. `usbfs_memory_mb` is set to low value
//!
//! If you use Linux, you may need to increase `usbfs_memory_mb` limit.
//! By default, USB-FS on Linux systems only allows 16 MB of buffer memory for all USB devices. This is quite low for high-resolution image streaming.
//! We recommend you to set the value to 1000MB. You could set the value as following:
//! ```sh
//! echo 1000 > /sys/module/usbcore/parameters/usbfs_memory_mb
//! ```
//!
//! ## License
//! This project is licenced under [MPL 2.0][license].
//!
//! [license]: https://github.com/cameleon-rs/cameleon/blob/main/LICENSE

#![warn(missing_docs)]
#![allow(
    clippy::similar_names,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions
)]

pub mod camera;
pub mod genapi;
pub mod payload;
#[cfg(feature = "libusb")]
pub mod u3v;

pub use camera::{Camera, CameraInfo, DeviceControl, PayloadStream};

use std::{borrow::Cow, num::TryFromIntError};

/// A specialized `Result` type for `camera::Camera`.
pub type CameleonResult<T> = std::result::Result<T, CameleonError>;

/// An error type returned from the `camera::Camera`.
#[derive(Debug, thiserror::Error)]
pub enum CameleonError {
    /// An error from device control.
    #[error("control error: {0}")]
    ControlError(#[from] ControlError),

    /// An error from payload stream.
    #[error("stream error: {0}")]
    StreamError(#[from] StreamError),

    /// `GenApi` context is not laoded yet.
    #[error("`GenApi` context is missing")]
    GenApiContextMissing,

    /// `GenApi` xml doesn't meet `GenApi SFNC` specification.
    #[error("invalid `GenApi` xml: {0}")]
    InvalidGenApiXml(Cow<'static, str>),

    /// An error when `GenApi` node operation failed.
    #[error("`GenApi` error: {0}")]
    GenApiError(#[from] cameleon_genapi::GenApiError),
}

/// A specialized `Result` type for device control.
pub type ControlResult<T> = std::result::Result<T, ControlError>;

/// An error detail for communication with a device.
#[derive(Debug, thiserror::Error)]
pub enum DeviceIoError {
    /// Underlying transport error.
    #[cfg(feature = "libusb")]
    #[error(transparent)]
    Transport(#[from] cameleon_device::u3v::Error),

    /// Protocol or runtime error represented as a message.
    #[error("{0}")]
    Message(Cow<'static, str>),
}

impl DeviceIoError {
    /// Constructs [`DeviceIoError::Message`].
    #[must_use]
    pub fn msg(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::Message(msg.into())
    }
}

/// An error type for device control.
#[derive(Debug, thiserror::Error)]
pub enum ControlError {
    /// The device is busy, may be opened by another application.
    #[error("device is busy")]
    Busy,

    /// The device is disconnected from the host.
    #[error("device is disconnected")]
    Disconnected,

    /// IO error.
    #[error("input/output error: {0}")]
    Io(DeviceIoError),

    /// Timeout has occured when receiving stream payload.
    #[error("timeout has occured when receiving stream payload")]
    Timeout,

    /// The device is not opened.
    #[error("device is not opened")]
    NotOpened,

    /// The device doesn't follow the spceicifation.
    #[error("invalid device: {0}")]
    InvalidDevice(Cow<'static, str>),

    /// Buffer is too small to receive data.
    #[error("buffer is too small to recieve data")]
    BufferTooSmall,

    /// Try to write invalid data to the device, or received data from the device is semantically invalid.
    /// e.g. try to write too large data that will overrun register.
    #[error("try to write invalid data to the device: {0}")]
    InvalidData(Box<dyn std::error::Error + Send + Sync>),
}

/// A specialized `Result` type for streaming.
pub type StreamResult<T> = std::result::Result<T, StreamError>;

/// An error type related to payload streaming.
#[derive(Debug, thiserror::Error)]
pub enum StreamError {
    /// Failed to receive [`payload::Payload`].
    #[error("failed to receive payload: {0}")]
    ReceiveError(Cow<'static, str>),

    /// Failed to send [`payload::Payload`].
    #[error("failed to send payload: {0}")]
    SendError(Cow<'static, str>),

    /// Payload leader is invalid.
    #[error("invalid payload has been sent: {0}")]
    InvalidPayload(Cow<'static, str>),

    /// The device is disconnected from the host.
    #[error("device is disconnected")]
    Disconnected,

    /// IO error.
    #[error("can't communicate with the device: {0}")]
    Io(DeviceIoError),

    /// Timeout has occured when receiving stream payload.
    #[error("timeout has occured when receiving stream payload")]
    Timeout,

    /// A panic has occurred in streaming loop.
    #[error("a panic has occurred in streaming loop: {0}")]
    Poisoned(Cow<'static, str>),

    /// Buffer is too small to receive data.
    #[error("buffer is too small to recieve data")]
    BufferTooSmall,

    /// Streaming is already started.
    #[error(
        "streaming is already started. can't use the handle from the outside of streaming loop"
    )]
    InStreaming,
}

impl From<TryFromIntError> for ControlError {
    fn from(e: TryFromIntError) -> Self {
        Self::InvalidDevice(format!("internal data has invalid num type: {e}").into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Taken from https://stackoverflow.com/a/32765782/4345715
    // Can be replaced by https://crates.io/crates/static_assertions
    const _: () = {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        #[expect(unused)]
        fn assert_cameleon_error_is_send_sync() {
            assert_send::<CameleonError>();
            assert_sync::<CameleonError>();
        }
    };
}
