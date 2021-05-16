//! This module contains types that is the main entry types of the `Cameleon`.
use auto_impl::auto_impl;
use tracing::info;

use super::{
    genapi::{DefaultGenApiCtxt, FromXml, GenApiCtxt, ParamsCtxt},
    payload::{channel, PayloadReceiver, PayloadSender},
    CameleonError, CameleonResult, ControlResult, StreamError, StreamResult,
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

macro_rules! expect_node {
    ($ctxt:expr, $name:literal, $as_type:ident) => {{
        let err_msg = std::concat!("missing ", $name);
        let err_msg2 = std::concat!($name, " has invalid interface");
        $ctxt
            .node($name)
            .ok_or_else(|| CameleonError::InvalidGenApiXml(err_msg.into()))?
            .$as_type(&mut $ctxt)
            .ok_or_else(|| CameleonError::InvalidGenApiXml(err_msg2.into()))?
    }};
}

impl<Ctrl, Strm, Ctxt> Camera<Ctrl, Strm, Ctxt> {
    /// Opens the camera. Ensure calling this method before start using the camera.
    #[tracing::instrument(skip(self),
                          level = "info",
                          fields(camera = ?self.info()))]
    pub fn open(&mut self) -> CameleonResult<()>
    where
        Ctrl: DeviceControl,
        Strm: PayloadStream,
    {
        info!("try opening the device");
        self.ctrl.open()?;
        self.strm.open()?;
        info!("opened the device successfully");
        Ok(())
    }

    /// Closes the camera.  
    /// Make sure to call this method before the camera is dropped.
    /// This method is NOT automatically called in `Drop` for flexibility.
    #[tracing::instrument(skip(self),
                          level = "info",
                          fields(camera = ?self.info()))]
    pub fn close(&mut self) -> CameleonResult<()>
    where
        Ctrl: DeviceControl,
        Strm: PayloadStream,
        Ctxt: GenApiCtxt,
    {
        info!("try closing the device");
        self.stop_streaming()?;
        self.ctrl.close()?;
        self.strm.close()?;
        if let Some(ctxt) = &mut self.ctxt {
            ctxt.clear_cache()
        }
        info!("closed the device successfully");
        Ok(())
    }

    /// Loads `GenApi` xml from the device and builds the context, then returns the `GenApi` xml
    /// string.  
    /// Once the context has been built, the string itself is no longer needed. Therefore, you can
    /// drop the returned string at any time.
    pub fn load_context(&mut self) -> CameleonResult<String>
    where
        Ctrl: DeviceControl,
        Strm: PayloadStream,
        Ctxt: GenApiCtxt + FromXml,
    {
        let xml = self.ctrl.genapi()?;
        self.ctxt = Some(Ctxt::from_xml(&xml)?);
        Ok(xml)
    }

    /// Starts streaming and returns the receiver for the `Payload`.
    /// Make sure to load `GenApi` context before calling this method.
    /// See [`Self::load_context`] and [`Self::set_context`] how to configure `GenApi` context.
    ///
    /// NOTE: This method doesn't change `AcquisitionMode` which defined in `GenICam SFNC`.  
    /// We recommend you to set the node to `Continuous` if you don't know which mode is the best.
    /// See the `GenICam SFNC` specification for more details.
    ///
    /// # Arguments
    ///
    /// * `cap` - A capacity of the paylaod receiver, the sender will stop to send a payload when it
    /// gets full.
    ///
    /// # Panics
    /// If `cap` is zero, this method will panic.
    #[tracing::instrument(skip(self),
                          level = "info",
                          fields(camera = ?self.info()))]
    pub fn start_streaming(&mut self, cap: usize) -> CameleonResult<PayloadReceiver>
    where
        Ctrl: DeviceControl<StrmParams = Strm::StrmParams>,
        Strm: PayloadStream,
        Ctxt: GenApiCtxt,
    {
        const DEFAULT_BUFFER_CAP: usize = 5;
        info!("try starting streaming");

        if self.strm.is_loop_running() {
            return Err(StreamError::InStreaming.into());
        }
        self.ctrl.enable_streaming()?;

        let mut ctxt = self.params_ctxt()?;
        expect_node!(ctxt, "TLParamsLocked", as_integer).set_value(&mut ctxt, 1)?;
        expect_node!(ctxt, "AcquisitionStart", as_command).execute(&mut ctxt)?;

        let strm_params = self.ctrl.enable_streaming()?;
        let (sender, receiver) = channel(cap, DEFAULT_BUFFER_CAP);
        self.strm.run_streaming_loop(sender, strm_params)?;

        info!("start streaming successfully");
        Ok(receiver)
    }

    /// Stops the streaming. The receiver returned from the previous [`Self::start_streaming`]
    /// call will be invalidated.
    #[tracing::instrument(skip(self),
                          level = "info",
                          fields(camera = ?self.info()))]
    pub fn stop_streaming(&mut self) -> CameleonResult<()>
    where
        Ctrl: DeviceControl,
        Strm: PayloadStream,
        Ctxt: GenApiCtxt,
    {
        info!("try stopping streaming");
        if !self.strm.is_loop_running() {
            return Ok(());
        }
        let mut ctxt = self.params_ctxt()?;
        expect_node!(ctxt, "AcquisitionStop", as_command).execute(&mut ctxt)?;
        expect_node!(ctxt, "TLParamsLocked", as_integer).set_value(&mut ctxt, 0)?;
        self.ctrl.disable_streaming()?;
        info!("stop streaming successfully");
        Ok(())
    }

    /// Returns the context of the camera params.
    /// Make sure to load `GenApi` context before calling this method.
    pub fn params_ctxt(&mut self) -> CameleonResult<ParamsCtxt<&mut Ctrl, &mut Ctxt>>
    where
        Ctrl: DeviceControl,
        Strm: PayloadStream,
        Ctxt: GenApiCtxt,
    {
        if let Some(ctxt) = self.ctxt.as_mut() {
            Ok(ParamsCtxt {
                ctrl: &mut self.ctrl,
                ctxt,
            })
        } else {
            Err(CameleonError::GenApiContextMissing)
        }
    }

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

    /// Set a context to the camera. It's recommended to use [`Self::load_context`] instead if `Self::Ctxt`
    /// implements [`FromXml`] trait.
    pub fn set_context<Ctxt2>(self, ctxt: Ctxt2) -> Camera<Ctrl, Strm, Ctxt2> {
        Camera {
            ctrl: self.ctrl,
            strm: self.strm,
            ctxt: Some(ctxt),
            info: self.info,
        }
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
    fn genapi(&mut self) -> ControlResult<String>;

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
