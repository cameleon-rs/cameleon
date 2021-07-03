/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![doc(hidden)]
//! This module contains libusb async api wrapper without any overhead.
//! NEVER make this module public because all functions in this module may cause UB if
//! preconditions are not followed.
// The implementation in the module is written with heavily reference to
// https://github.com/kevinmehall/rusb/blob/km-pipe-approach/src/device_handle/async_api.rs.

use std::{
    collections::VecDeque,
    convert::TryInto,
    ptr::NonNull,
    sync::atomic::{AtomicBool, Ordering::SeqCst},
    time::{Duration, Instant},
};

use super::{
    channel::ReceiveIfaceInfo, device::RusbDeviceHandle, LibUsbError, ReceiveChannel, Result,
};
use rusb::UsbContext;

#[doc(hidden)]
/// Represents a pool of asynchronous transfers, that can be polled to completion.
pub struct AsyncPool<'a> {
    handle: AsyncHandle<'a>,
    iface_info: ReceiveIfaceInfo,
    pending: VecDeque<AsyncTransfer>,
}

impl<'a> AsyncPool<'a> {
    #[doc(hidden)]
    pub fn new(channel: &'a ReceiveChannel) -> Self {
        let iface_info = channel.iface_info.clone();
        let handle = get_handle(channel);
        Self {
            handle,
            iface_info,
            pending: VecDeque::new(),
        }
    }

    #[doc(hidden)]
    pub fn submit(&mut self, buf: &mut [u8]) -> Result<()> {
        // Safety: If transfer is submitted, it is pushed onto `pending` where it will be
        // dropped before `device` is freed.
        unsafe {
            let mut transfer =
                AsyncTransfer::new_bulk(self.handle.as_raw(), self.iface_info.bulk_in_ep, buf);
            transfer.submit()?;
            self.pending.push_back(transfer);
            Ok(())
        }
    }

    #[doc(hidden)]
    /// # Panics
    ///
    /// Panics if there is no pending transfer.
    pub fn poll(&mut self, timeout: Duration) -> Result<usize> {
        debug_assert!(!self.pending.is_empty());
        let next = self.pending.front().unwrap();
        if poll_completed(self.handle.context(), timeout, next.completed_flag())? {
            let mut transfer = self.pending.pop_front().unwrap();
            Ok(transfer.handle_completed()?)
        } else {
            Err(LibUsbError::Timeout.into())
        }
    }

    #[doc(hidden)]
    pub fn cancel_all(&mut self) {
        // Cancel in reverse order to avoid a race condition in which one
        // transfer is cancelled but another submitted later makes its way onto
        // the bus.
        for transfer in self.pending.iter_mut().rev() {
            transfer.cancel();
        }
    }

    /// Returns the number of async transfers pending.
    #[doc(hidden)]
    pub fn pending(&self) -> usize {
        self.pending.len()
    }

    /// Returns `true` if there is no pending transfer.
    #[doc(hidden)]
    pub fn is_empty(&self) -> bool {
        self.pending() == 0
    }
}

impl<'a> Drop for AsyncPool<'a> {
    fn drop(&mut self) {
        self.cancel_all();
        while !self.is_empty() {
            self.poll(Duration::from_secs(1)).ok();
        }
    }
}

struct AsyncTransfer {
    ptr: NonNull<libusb1_sys::libusb_transfer>,
}

impl AsyncTransfer {
    /// Invariant: Caller must ensure `device` outlives this transfer.
    unsafe fn new_bulk(
        device: *mut libusb1_sys::libusb_device_handle,
        endpoint: u8,
        buffer: &mut [u8],
    ) -> Self {
        // non-isochronous endpoints (e.g. control, bulk, interrupt) specify a value of 0
        // This is step 1 of async API
        let ptr = libusb1_sys::libusb_alloc_transfer(0);
        let ptr = NonNull::new(ptr).expect("Could not allocate transfer!");

        let user_data = Box::into_raw(Box::new(AtomicBool::new(false))).cast::<libc::c_void>();

        let length = buffer.len() as libc::c_int;

        libusb1_sys::libusb_fill_bulk_transfer(
            ptr.as_ptr(),
            device,
            endpoint,
            buffer.as_ptr() as *mut u8,
            length,
            Self::transfer_cb,
            user_data,
            0,
        );

        Self { ptr }
    }

    //// Part of step 4 of async API the transfer is finished being handled when
    //// `poll()` is called.
    extern "system" fn transfer_cb(transfer: *mut libusb1_sys::libusb_transfer) {
        // Safety: transfer is still valid because libusb just completed
        // it but we haven't told anyone yet. user_data remains valid
        // because it is freed only with the transfer.
        // After the store to completed, these may no longer be valid if
        // the polling thread freed it after seeing it completed.
        let completed = unsafe {
            let transfer = &mut *transfer;
            &*transfer.user_data.cast::<AtomicBool>()
        };
        completed.store(true, SeqCst);
    }

    fn transfer(&self) -> &libusb1_sys::libusb_transfer {
        // Safety: transfer remains valid as long as self
        unsafe { self.ptr.as_ref() }
    }

    fn completed_flag(&self) -> &AtomicBool {
        // Safety: transfer and user_data remain valid as long as self
        unsafe { &*self.transfer().user_data.cast::<AtomicBool>() }
    }

    // Step 3 of async API
    fn submit(&mut self) -> Result<()> {
        self.completed_flag().store(false, SeqCst);
        let errno = unsafe { libusb1_sys::libusb_submit_transfer(self.ptr.as_ptr()) };
        Ok(LibUsbError::from_libusb_error(errno)?)
    }

    fn cancel(&mut self) {
        unsafe {
            libusb1_sys::libusb_cancel_transfer(self.ptr.as_ptr());
        }
    }

    fn handle_completed(&mut self) -> Result<usize> {
        assert!(self
            .completed_flag()
            .load(std::sync::atomic::Ordering::Relaxed));
        use libusb1_sys::constants::*;
        let err = match self.transfer().status {
            LIBUSB_TRANSFER_COMPLETED => {
                let transfer = self.transfer();
                debug_assert!(transfer.length >= transfer.actual_length);
                return Ok(transfer.actual_length as usize);
            }
            LIBUSB_TRANSFER_CANCELLED => LibUsbError::Timeout,
            LIBUSB_TRANSFER_ERROR => LibUsbError::Other,
            LIBUSB_TRANSFER_TIMED_OUT => {
                unreachable!("We are using timeout=0 which means no timeout")
            }
            LIBUSB_TRANSFER_STALL => LibUsbError::Pipe,
            LIBUSB_TRANSFER_NO_DEVICE => LibUsbError::NoDevice,
            LIBUSB_TRANSFER_OVERFLOW => LibUsbError::Overflow,
            _ => unreachable!(),
        };
        Err(err.into())
    }
}

/// Invariant: transfer must not be pending.
impl Drop for AsyncTransfer {
    fn drop(&mut self) {
        unsafe {
            libusb1_sys::libusb_free_transfer(self.ptr.as_ptr());
        }
    }
}

/// This is effectively libusb_handle_events_timeout_completed, but with
/// `completed` as `AtomicBool` instead of `c_int` so it is safe to access
/// without the events lock held. It also continues polling until completion,
/// timeout, or error, instead of potentially returning early.
///
/// This design is based on
/// https://libusb.sourceforge.io/api-1.0/libusb_mtasync.html#threadwait
fn poll_completed(
    ctx: &impl UsbContext,
    timeout: Duration,
    completed: &AtomicBool,
) -> Result<bool> {
    use libusb1_sys::{constants::*, *};

    let deadline = Instant::now() + timeout;

    unsafe {
        let mut err = 0_i32;
        while err == 0_i32 && !completed.load(SeqCst) && deadline > Instant::now() {
            let remaining = deadline.saturating_duration_since(Instant::now());
            let timeval = libc::timeval {
                tv_sec: remaining.as_secs().try_into().unwrap(),
                tv_usec: remaining.subsec_micros().try_into().unwrap(),
            };

            if libusb_try_lock_events(ctx.as_raw()) == 0_i32 {
                if !completed.load(SeqCst) && libusb_event_handling_ok(ctx.as_raw()) != 0_i32 {
                    err = libusb_handle_events_locked(ctx.as_raw(), &timeval as *const _);
                }
                libusb_unlock_events(ctx.as_raw());
            } else {
                libusb_lock_event_waiters(ctx.as_raw());
                if !completed.load(SeqCst) && libusb_event_handler_active(ctx.as_raw()) != 0_i32 {
                    libusb_wait_for_event(ctx.as_raw(), &timeval as *const _);
                }
                libusb_unlock_event_waiters(ctx.as_raw());
            }
        }

        match err {
            0_i32 => Ok(completed.load(SeqCst)),
            LIBUSB_ERROR_TIMEOUT => Ok(false),
            e => Err(LibUsbError::from_libusb_error(e).unwrap_err().into()),
        }
    }
}

impl LibUsbError {
    fn from_libusb_error(err: i32) -> std::result::Result<(), Self> {
        match err {
            0_i32 => Ok(()),
            libusb1_sys::constants::LIBUSB_ERROR_IO => Err(Self::Io),
            libusb1_sys::constants::LIBUSB_ERROR_INVALID_PARAM => Err(Self::InvalidParam),
            libusb1_sys::constants::LIBUSB_ERROR_ACCESS => Err(Self::Access),
            libusb1_sys::constants::LIBUSB_ERROR_NO_DEVICE => Err(Self::NoDevice),
            libusb1_sys::constants::LIBUSB_ERROR_NOT_FOUND => Err(Self::NotFound),
            libusb1_sys::constants::LIBUSB_ERROR_BUSY => Err(Self::Busy),
            libusb1_sys::constants::LIBUSB_ERROR_TIMEOUT => Err(Self::Timeout),
            libusb1_sys::constants::LIBUSB_ERROR_OVERFLOW => Err(Self::Overflow),
            libusb1_sys::constants::LIBUSB_ERROR_PIPE => Err(Self::Pipe),
            libusb1_sys::constants::LIBUSB_ERROR_INTERRUPTED => Err(Self::Interrupted),
            libusb1_sys::constants::LIBUSB_ERROR_NO_MEM => Err(Self::NoMem),
            libusb1_sys::constants::LIBUSB_ERROR_NOT_SUPPORTED => Err(Self::NotSupported),
            libusb1_sys::constants::LIBUSB_ERROR_OTHER => Err(Self::Other),
            _ => unreachable!(),
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {
        use std::sync::MutexGuard;
        struct AsyncHandle<'a>(MutexGuard<'a, Option<RusbDeviceHandle>>);

        impl<'a> AsyncHandle<'a> {
            fn context(&self) -> &impl UsbContext {
                self.0.as_ref().unwrap().context()
            }
            fn as_raw(&self) -> *mut libusb1_sys::libusb_device_handle {
                self.0.as_ref().unwrap().as_raw()
            }
        }

        fn get_handle(channel: &ReceiveChannel) -> AsyncHandle {
            AsyncHandle(channel.device_handle.handle.lock().unwrap())
        }
    } else {
        type AsyncHandle<'a> = &'a RusbDeviceHandle;

        fn get_handle(channel: &ReceiveChannel) -> AsyncHandle {
            &channel.device_handle
        }
    }
}
