//! This example describes how to use internal type conversions in `Camera`.
//! See also `cameleon/examples/custom_ctxt.rs` that describes the more advanced use of type conversions

use cameleon::genapi::{DefaultGenApiCtxt, NoCacheGenApiCtxt};
use cameleon::u3v::{enumerate_cameras, ControlHandle, StreamHandle};
use cameleon::Camera;

fn main() {
    // Enumerates cameras connected to the host.
    let mut cameras = enumerate_cameras().unwrap();
    if cameras.is_empty() {
        println!("no camera found!");
        return;
    }

    let camera: Camera<ControlHandle, StreamHandle, DefaultGenApiCtxt> = cameras.pop().unwrap();
    // Converts `DefaultGenApiCtxt` to `NoCacheGenApiCtxt`, this camera no more cache any
    // parameters from now on.
    let _camera: Camera<ControlHandle, StreamHandle, NoCacheGenApiCtxt> = camera.convert_into();
}
