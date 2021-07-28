/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module contains types that is the main entry types of the `Cameleon`.
//!
//! # Examples
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

use auto_impl::auto_impl;
use tracing::info;

use super::{
    genapi::{DefaultGenApiCtxt, FromXml, GenApiCtxt, ParamsCtxt},
    payload::{channel, PayloadReceiver, PayloadSender},
    CameleonError, CameleonResult, ControlResult, StreamError, StreamResult,
};

/// Provides easy-to-use access to a `GenICam` compatible camera.
///
/// # Examples
/// ```rust
/// use cameleon::u3v;
///
/// // Enumerates all cameras connected to the host.
/// let mut cameras = u3v::enumerate_cameras().unwrap();
/// if cameras.is_empty() {
///     println!("no camera found");
///     return;
/// }
/// let mut camera = cameras.pop().unwrap();
///
/// // Opens the camera.
/// camera.open().unwrap();
/// // Loads `GenApi` context. This is necessary for streaming.
/// camera.load_context().unwrap();
///
/// // Start streaming. Channel capacity is set to 3.
/// let payload_rx = camera.start_streaming(3).unwrap();
///
/// let mut payload_count = 0;
/// while payload_count < 10 {
///     match payload_rx.try_recv() {
///         Ok(payload) => {
///             println!(
///                 "payload received! block_id: {:?}, timestamp: {:?}",
///                 payload.id(),
///                 payload.timestamp()
///             );
///             if let Some(image_info) = payload.image_info() {
///                 println!("{:?}\n", image_info);
///                 let image = payload.image();
///                 // do something with the image.
///                 // ...
///             }
///             payload_count += 1;
///
///             // Send back payload to streaming loop to reuse the buffer. This is optional.
///             payload_rx.send_back(payload);
///         }
///         Err(_err) => {
///             continue;
///         }
///     }
/// }
///
/// // Closes the camera.
/// camera.close().unwrap();
/// ```
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
    ($ctxt:expr, $name:expr, $as_type:ident) => {{
        let err_msg = std::concat!("missing ", $name);
        let err_msg2 = std::concat!($name, " has invalid interface");
        $ctxt
            .node($name)
            .ok_or_else(|| CameleonError::InvalidGenApiXml(err_msg.into()))?
            .$as_type($ctxt)
            .ok_or_else(|| CameleonError::InvalidGenApiXml(err_msg2.into()))?
    }};
}

impl<Ctrl, Strm, Ctxt> Camera<Ctrl, Strm, Ctxt> {
    /// Opens the camera. Ensure calling this method before starting to use the camera.  
    ///
    /// See also [`close`](Self::close) which must be called when an opened camera is no more needed.
    ///
    /// # Examples
    /// ```rust
    /// # use cameleon::u3v;
    /// # let mut cameras = u3v::enumerate_cameras().unwrap();
    /// # if cameras.is_empty() {
    /// #     return;
    /// # }
    /// # let mut camera = cameras.pop().unwrap();
    /// // Opens the camera before using it.
    /// camera.open().unwrap();
    /// // .. Do something with camera.
    /// // Closes the camera after using it.
    /// camera.close().unwrap();
    /// ```
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
    ///
    /// Make sure to call this method before the camera is dropped.
    /// To keep flexibility, this method is NOT automatically called when `Camera::drop` is calles.
    ///
    /// # Examples
    /// ```rust
    /// # use cameleon::u3v;
    /// # let mut cameras = u3v::enumerate_cameras().unwrap();
    /// # if cameras.is_empty() {
    /// #     return;
    /// # }
    /// # let mut camera = cameras.pop().unwrap();
    /// // Opens the camera before using it.
    /// camera.open().unwrap();
    /// // .. Do something with camera.
    /// // Closes the camera after using it.
    /// camera.close().unwrap();
    /// ```
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
    ///
    /// Once the context has been built, the string itself is no longer needed. Therefore, you can
    /// drop the returned string at any time.
    ///
    /// # Examples
    /// ```rust
    /// // Enumerates all cameras connected to the host.
    /// # use cameleon::u3v;
    /// # let mut cameras = u3v::enumerate_cameras().unwrap();
    /// # if cameras.is_empty() {
    /// #     return;
    /// # }
    /// # let mut camera = cameras.pop().unwrap();
    /// // Opens the camera before using it.
    /// camera.open().unwrap();
    ///
    /// // Loads context. This enables you to edit parameters of the camera and start payload streaming.
    /// camera.load_context().unwrap();
    ///
    /// // Closes the camera.
    /// camera.close().unwrap();
    /// ```
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
    ///
    /// Make sure to load `GenApi` context before calling this method.
    /// See [`load_context`](Self::load_context) and [`set_context`](Self::set_context) how to configure `GenApi` context.
    ///
    /// NOTE: This method doesn't change `AcquisitionMode` which defined in `GenICam SFNC`.  
    /// We recommend you to set the node to `Continuous` if you don't know which mode is the best.
    ///
    /// See the `GenICam SFNC` specification for more details.
    ///
    /// # Examples
    /// ```rust
    /// # use cameleon::u3v;
    /// # let mut cameras = u3v::enumerate_cameras().unwrap();
    /// # if cameras.is_empty() {
    /// #     return;
    /// # }
    /// # let mut camera = cameras.pop().unwrap();
    /// camera.open().unwrap();
    /// camera.load_context().unwrap();
    ///
    /// // Start streaming. Channel capacity is set to 3.
    /// let payload_rx = camera.start_streaming(3).unwrap();
    /// // The streamed payload can be received like below:
    /// // payload_rx.recv().await.unwrap() or
    /// // payload.rx.try_recv().unwrap();
    ///
    /// // Closes the camera.
    /// camera.close().unwrap();
    /// ```
    ///
    /// # Arguments
    /// * `cap` - A capacity of the paylaod receiver, the sender will stop to send a payload when it
    /// gets full.
    ///
    ///
    /// # Panics
    /// If `cap` is zero, this method will panic.
    #[tracing::instrument(skip(self),
                          level = "info",
                          fields(camera = ?self.info()))]
    pub fn start_streaming(&mut self, cap: usize) -> CameleonResult<PayloadReceiver>
    where
        Ctrl: DeviceControl,
        Strm: PayloadStream,
        Ctxt: GenApiCtxt,
    {
        const DEFAULT_BUFFER_CAP: usize = 5;
        info!("try starting streaming");

        if self.strm.is_loop_running() {
            return Err(StreamError::InStreaming.into());
        }

        // Enable streaimng.
        self.ctrl.enable_streaming()?;
        let mut ctxt = self.params_ctxt()?;
        expect_node!(&ctxt, "TLParamsLocked", as_integer).set_value(&mut ctxt, 1)?;
        expect_node!(&ctxt, "AcquisitionStart", as_command).execute(&mut ctxt)?;

        // Start streaming loop.
        let (sender, receiver) = channel(cap, DEFAULT_BUFFER_CAP);
        self.strm.start_streaming_loop(sender, &mut self.ctrl)?;

        info!("start streaming successfully");
        Ok(receiver)
    }

    /// Stops the streaming.
    ///
    /// The receiver returned from the previous [`Self::start_streaming`]
    /// call will be invalidated.
    ///
    /// This method is automatically called in [`close`](Self::close), so no need to call
    /// explicitly when you close the camera.
    ///
    /// # Examples
    /// ```
    /// # use cameleon::u3v;
    /// # let mut cameras = u3v::enumerate_cameras().unwrap();
    /// # if cameras.is_empty() {
    /// #     return;
    /// # }
    /// # let mut camera = cameras.pop().unwrap();
    /// camera.open().unwrap();
    /// // Loads `GenApi` context. This is necessary for streaming.
    /// camera.load_context().unwrap();
    ///
    /// // Start streaming. Channel capacity is set to 3.
    /// let payload_rx = camera.start_streaming(3).unwrap();
    ///
    /// camera.stop_streaming().unwrap();
    /// ```
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

        // Stop streaming loop.
        self.strm.stop_streaming_loop()?;

        // Disable streaming.
        let mut ctxt = self.params_ctxt()?;
        expect_node!(&ctxt, "AcquisitionStop", as_command).execute(&mut ctxt)?;
        expect_node!(&ctxt, "TLParamsLocked", as_integer).set_value(&mut ctxt, 0)?;
        self.ctrl.disable_streaming()?;

        info!("stop streaming successfully");
        Ok(())
    }

    /// Returns the context of the camera params.
    ///
    /// Make sure to load `GenApi` context before calling this method.
    /// See [`load_context`](Self::load_context) and [`set_context`](Self::set_context) how to configure `GenApi` context.
    ///
    /// # Examples
    /// ```
    /// # use cameleon::u3v;
    /// # let mut cameras = u3v::enumerate_cameras().unwrap();
    /// # if cameras.is_empty() {
    /// #     return;
    /// # }
    /// # let mut camera = cameras.pop().unwrap();
    /// camera.open().unwrap();
    /// camera.load_context().unwrap();
    ///
    /// // Get params context.
    /// let mut params_ctxt = camera.params_ctxt().unwrap();
    ///
    /// // Get `Gain` node of `GenApi`.
    /// // `GenApi SFNC` defines that `Gain` node should have `IFloat` interface,
    /// // so this conversion would be success if the camera follows that.
    /// // Some vendors may define `Gain` node as `IInteger`, in that case, use
    /// // `as_integer(&params_ctxt)` instead of `as_float(&params_ctxt).
    /// let gain_node = params_ctxt.node("Gain").unwrap().as_float(&params_ctxt).unwrap();
    ///
    /// // Get the current value of `Gain`.
    /// if gain_node.is_readable(&mut params_ctxt).unwrap() {
    ///     let value = gain_node.value(&mut params_ctxt).unwrap();
    ///     println!("{}", value);
    /// }
    ///
    /// // Set `0.1` to `Gain`.
    /// if gain_node.is_writable(&mut params_ctxt).unwrap() {
    ///     gain_node.set_value(&mut params_ctxt, 0.1).unwrap();
    /// }
    /// # camera.close();
    /// ```
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

    /// Returns basic information of the camera.
    ///
    /// This information can be obtained without calling [`Self::open`].
    /// # Examples
    /// ```
    /// # use cameleon::u3v;
    /// # let mut cameras = u3v::enumerate_cameras().unwrap();
    /// # if cameras.is_empty() {
    /// #     return;
    /// # }
    /// # let mut camera = cameras.pop().unwrap();
    /// let info = camera.info();
    /// ```
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

    /// Converts internal types.
    ///
    /// This method works same as `std::convert::From`, just hack to avoid
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
    ///
    /// # Examples
    /// ```rust
    /// # use cameleon::u3v;
    /// # let mut cameras = u3v::enumerate_cameras().unwrap();
    /// # if cameras.is_empty() {
    /// #     return;
    /// # }
    /// # let camera = cameras.pop().unwrap();
    /// use cameleon::{DeviceControl, PayloadStream, Camera};
    /// use cameleon::genapi::NoCacheGenApiCtxt;
    ///
    /// // Convert into `Camera<Box<dyn DeviceControl>, Box<dyn PayloadStream>, NoCacheGenApiCtxt>`.
    /// let dyn_camera: Camera<Box<dyn DeviceControl>, Box<dyn PayloadStream>, NoCacheGenApiCtxt> =
    ///     camera.convert_into();
    /// ```
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
pub trait DeviceControl {
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

    /// Reads 4 bytes data from the address.
    fn read_reg(&mut self, address: u64) -> ControlResult<u32>;

    /// Writes data to the device's memory.
    fn write(&mut self, address: u64, data: &[u8]) -> ControlResult<()>;

    /// Writes 4 bytes data to the address.
    fn write_reg(&mut self, address: u64, data: u32) -> ControlResult<()>;

    /// Returns `GenICam` xml string.
    fn genapi(&mut self) -> ControlResult<String>;

    /// Enables streaming.
    fn enable_streaming(&mut self) -> ControlResult<()>;

    /// Disables streaming.
    fn disable_streaming(&mut self) -> ControlResult<()>;
}

/// This trait provides streaming capability.
#[auto_impl(&mut, Box)]
pub trait PayloadStream {
    /// Opens the handle.
    fn open(&mut self) -> StreamResult<()>;

    /// Closes the handle.
    fn close(&mut self) -> StreamResult<()>;

    /// Starts streaming.
    fn start_streaming_loop(
        &mut self,
        sender: PayloadSender,
        ctrl: &mut dyn DeviceControl,
    ) -> StreamResult<()>;

    /// Stops streaming.
    fn stop_streaming_loop(&mut self) -> StreamResult<()>;

    /// Returns `true` if streaming loop is running.
    fn is_loop_running(&self) -> bool;
}
