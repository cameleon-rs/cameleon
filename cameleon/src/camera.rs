//! This module contains types that is the main entry types of the `Cameleon`.
use auto_impl::auto_impl;
use tracing::info;

use super::{
    genapi::{DefaultGenApiCtxt, GenApiCtxt},
    payload::PayloadSender,
    CameleonResult, ControlResult, StreamResult,
};

/// Provides easy-to-use access to a `GenICam` compatible camera.
#[derive(Debug, Clone)]
pub struct Camera<Ctrl, Strm, Ctxt = DefaultGenApiCtxt> {
    /// Device control handle of the camera.
    pub ctrl: Ctrl,
    /// Payload stream handle of the camera.
    pub strm: Strm,
    /// `GenApi context` of the camera.
    pub ctxt: Option<Ctxt>,
    /// Information of the camera.
    info: CameraInfo,
}

impl<Ctrl, Strm, Ctxt> Camera<Ctrl, Strm, Ctxt>
where
    Ctrl: DeviceControl,
    Strm: PayloadStream,
    Ctxt: GenApiCtxt,
{
}

impl<Ctrl, Strm, Ctxt> Camera<Ctrl, Strm, Ctxt>
where
    Ctrl: DeviceControl,
    Strm: PayloadStream,
{
    /// Opens the camera. Ensure calling this method before start using the camera.
    #[tracing::instrument(skip(self),
                          level = "info",
                          fields(camera = ?self.info()))]
    pub fn open(&mut self) -> CameleonResult<()> {
        info!("try opening the device");
        self.ctrl.open()?;
        self.strm.open()?;
        info!("opened the device successfully");
        Ok(())
    }

    /// Closes the camera.
    #[tracing::instrument(skip(self),
                          level = "info",
                          fields(camera = ?self.info()))]
    pub fn close(&mut self) -> CameleonResult<()> {
        info!("try closing the device");
        self.ctrl.close()?;
        self.strm.close()?;
        info!("closed the device successfully");
        Ok(())
    }
}

impl<Ctrl, Strm, Ctxt> Camera<Ctrl, Strm, Ctxt> {
    /// Returns the information of the camera.
    /// This information can be use before calling [`Self::open`].
    pub fn info(&self) -> &CameraInfo {
        &self.info
    }

    /// Constructs a camera.
    pub fn new(ctrl: Ctrl, strm: Strm, ctxt: Option<Ctxt>, info: CameraInfo) -> Self {
        Self {
            ctrl,
            strm,
            ctxt,
            info,
        }
    }

    /// Converts internal types. This method work same as `std::convert::From`, just hack to avoid
    /// `E0119`.
    pub fn convert_from<Ctrl2, Strm2, Ctxt2>(from: Camera<Ctrl2, Strm2, Ctxt2>) -> Self
    where
        Ctrl: From<Ctrl2>,
        Strm: From<Strm2>,
        Ctxt: From<Ctxt2>,
    {
        Camera::new(
            from.ctrl.into(),
            from.strm.into(),
            from.ctxt.map(|ctxt| ctxt.into()),
            from.info,
        )
    }

    /// Converts internal types. This method work same as `std::convert::Into`, just hack to avoid
    /// `E0119`.
    pub fn convert_into<Ctrl2, Strm2, Ctxt2>(self) -> Camera<Ctrl2, Strm2, Ctxt2>
    where
        Ctrl: Into<Ctrl2>,
        Strm: Into<Strm2>,
        Ctxt: Into<Ctxt2>,
    {
        Camera::new(
            self.ctrl.into(),
            self.strm.into(),
            self.ctxt.map(|ctxt| ctxt.into()),
            self.info,
        )
    }
}

/// Information of the camera.
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct CameraInfo {
    /// Vendor name of the camera.
    pub vendor_name: String,
    /// Model name of the camera.
    pub model_name: String,
    ///Serial number of the camera.
    pub serial_number: String,
}

/// This trait provides operations on the device's memory.
#[auto_impl(&mut, Box)]
pub trait DeviceControl: cameleon_genapi::Device {
    /// A parameter type used for streaming.
    type StrmParams;

    /// Opens the handle.
    fn open(&mut self) -> ControlResult<()>;

    /// Closes the handle.
    fn close(&mut self) -> ControlResult<()>;

    /// Returns `true` if device is already opened.
    fn is_opened(&self) -> bool;

    /// Reads data from the device's memory.
    ///
    /// Reads length is same as `buf.len()`.
    fn read(&mut self, address: u64, buf: &mut [u8]) -> ControlResult<()>;

    /// Writes data to the device's memory.
    fn write(&mut self, address: u64, data: &[u8]) -> ControlResult<()>;

    /// Returns `GenICam` xml string.
    fn gen_api(&mut self) -> ControlResult<String>;

    /// Enables streaming.
    fn enable_streaming(&mut self) -> ControlResult<Self::StrmParams>;

    /// Disables streaming.
    fn disable_streaming(&mut self) -> ControlResult<()>;
}

/// This trait provides streaming capability.
#[auto_impl(&mut, Box)]
pub trait PayloadStream {
    /// A parameter type used for streaming.
    type StrmParams;

    /// Opens the handle.
    fn open(&mut self) -> StreamResult<()>;

    /// Closes the handle.
    fn close(&mut self) -> StreamResult<()>;

    /// Starts streaming.
    fn run_streaming_loop(
        &mut self,
        sender: PayloadSender,
        params: Self::StrmParams,
    ) -> StreamResult<()>;

    /// Stops streaming.
    fn stop_streaming_loop(&mut self) -> StreamResult<()>;

    /// Returns `true` if streaming loop is running.
    fn is_loop_running(&self) -> bool;
}
