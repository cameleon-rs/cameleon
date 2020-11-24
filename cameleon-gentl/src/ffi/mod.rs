#[macro_use]
mod macros;

use std::{
    cell::RefCell,
    sync::{Mutex, RwLock},
};

use crate::{imp::system::SystemModule, GenTlError, GenTlResult};

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct GC_ERROR(i32);

impl Into<GC_ERROR> for &GenTlError {
    fn into(self) -> GC_ERROR {
        use GenTlError::*;
        let code = match self {
            Error(..) => -1001,
            NotInitialized => -1002,
            NotImplemented => -1003,
            ResourceInUse => -1004,
            AccessDenied => -1005,
            InvalidHandle => -1006,
            InvalidId(..) => -1007,
            NoData => -1008,
            InvalidParameter => -1009,
            Io(..) => -1010,
            Timeout => -1011,
            Abort => -1012,
            InvalidBuffer => -1013,
            NotAvailable => -1014,
            InvalidAddress => -1015,
            BufferTooSmall => -1016,
            InvalidIndex => -1017,
            ParsingChunkData => -1018,
            InvalidValue(..) => -1019,
            ResourceExhausted => -1020,
            OutOfMemory => -1021,
            Busy => -1022,
            Ambiguous => -1023,
        };
        GC_ERROR(code)
    }
}

impl Into<GC_ERROR> for GenTlError {
    fn into(self) -> GC_ERROR {
        (&self).into()
    }
}

impl<T> Into<GC_ERROR> for GenTlResult<T> {
    fn into(self) -> GC_ERROR {
        match self {
            Ok(..) => GC_ERROR(0),
            Err(e) => e.into(),
        }
    }
}

impl<T> Into<GC_ERROR> for &GenTlResult<T> {
    fn into(self) -> GC_ERROR {
        match self {
            Ok(..) => GC_ERROR(0),
            Err(e) => e.into(),
        }
    }
}

struct LastError {
    err: Option<GenTlError>,
}

enum HandleType {
    System(Mutex<SystemModule>),
}

#[repr(C)]
#[allow(dead_code)]

newtype_enum! {
    pub enum INFO_DATATYPE {
        INFO_DATATYPE_UNKNOWN = 0,
        INFO_DATATYPE_STRING = 1,
        INFO_DATATYPE_STRINGLIST = 2,
        INFO_DATATYPE_INT16 = 3,
        INFO_DATATYPE_UINT16 = 4,
        INFO_DATATYPE_INT32 = 5,
        INFO_DATATYPE_UINT32 = 6,
        INFO_DATATYPE_INT64 = 7,
        INFO_DATATYPE_UINT64 = 8,
        INFO_DATATYPE_FLOAT64 = 9,
        INFO_DATATYPE_PTR = 10,
        INFO_DATATYPE_BOOL8 = 11,
        INFO_DATATYPE_SIZET = 12,
        INFO_DATATYPE_BUFFER = 13,
        INFO_DATATYPE_PTRDIFF = 14,
    }
}

lazy_static::lazy_static! {
    static ref IS_MODULE_INITIALIZED: RwLock<bool> = RwLock::new(false);
}

thread_local! {
    static LAST_ERROR: RefCell<LastError> = {
        let last_error = LastError {
            err: None,
        };
        RefCell::new(last_error)
    }
}

fn save_last_error<T>(res: GenTlResult<T>) {
    match res {
        Ok(_) => {}
        Err(e) => LAST_ERROR.with(|err| {
            let mut err = err.borrow_mut();
            err.err = Some(e);
        }),
    }
}

fn copy_str(src: &str, dst: *mut libc::c_char) -> GenTlResult<libc::size_t> {
    if !src.is_ascii() {
        return Err(GenTlError::InvalidValue("string is not ascii".into()));
    }

    let len_without_null = src.len();
    if !dst.is_null() {
        unsafe {
            std::ptr::copy_nonoverlapping(
                src.as_ptr() as *const libc::c_char,
                dst,
                len_without_null,
            );
            dst.offset(len_without_null as isize).write(0); // Null terminated.
        }
    }

    Ok(len_without_null + 1)
}

fn assert_lib_initialized() -> GenTlResult<()> {
    if *IS_MODULE_INITIALIZED.read().unwrap() {
        Ok(())
    } else {
        Err(GenTlError::NotInitialized)
    }
}

gentl_api!(
    no_assert pub fn GCInitLib() -> GenTlResult<()> {
        let mut is_init = IS_MODULE_INITIALIZED.write().unwrap();

        if *is_init {
            Err(GenTlError::ResourceInUse)
        } else {
            *is_init = true;
            Ok(())
        }
    }
);

gentl_api!(
    pub fn GCCloseLib() -> GenTlResult<()> {
        let mut is_init = IS_MODULE_INITIALIZED.write().unwrap();
        if !*is_init {
            Err(GenTlError::NotInitialized)
        } else {
            *is_init = false;
            Ok(())
        }
    }
);

gentl_api!(
    pub fn CGCGetInfo(
        _iInfoCmd: i32,
        _piType: i32,
        _pBuffer: *mut libc::c_void,
        _piSize: *mut libc::size_t,
    ) -> GenTlResult<()> {
        Err(GenTlError::NotImplemented)
    }
);

gentl_api!(
    pub fn GCGetLastError(
        piErrorCode: *mut GC_ERROR,
        sErrorText: *mut libc::c_char,
        piSize: *mut libc::size_t,
    ) -> GenTlResult<()> {
        let (code, size) = match LAST_ERROR.with(|err| {
            let err = err.borrow();
            match &err.err {
                Some(err) => Some((err.into(), format!("{}", err))),
                _ => None,
            }
        }) {
            Some((code, text)) => {
                let size = copy_str(&text, sErrorText)?;
                (code, size)
            }
            None => {
                let size = copy_str("No Error", sErrorText)?;
                (Ok(()).into(), size)
            }
        };

        unsafe {
            *piErrorCode = code;
            *piSize = size;
        }

        Ok(())
    }
);
