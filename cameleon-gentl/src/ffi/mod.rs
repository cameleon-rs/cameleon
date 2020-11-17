use std::{borrow::Cow, cell::RefCell, sync::RwLock};

use crate::imp::{GenTlError, GenTlResult};

#[allow(unused)]
#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum GCErrorKind {
    GC_ERR_SUCCESS = 0,
    GC_ERR_ERROR = -1001,
    GC_ERR_NOT_INITIALIZED = -1002,
    GC_ERR_NOT_IMPLEMENTED = -1003,
    GC_ERR_RESOURCE_IN_USE = -1004,
    GC_ERR_ACCESS_DENIED = -1005,
    GC_ERR_INVALID_HANDLE = -1006,
    GC_ERR_INVALID_ID = -1007,
    GC_ERR_NO_DATA = -1008,
    GC_ERR_INVALID_PARAMETER = -1009,
    GC_ERR_IO = -1010,
    GC_ERR_TIMEOUT = -1011,
    GC_ERR_ABORT = -1012,
    GC_ERR_INVALID_BUFFER = -1013,
    GC_ERR_NOT_AVAILABLE = -1014,
    GC_ERR_INVALID_ADDRESS = -1015,
    GC_ERR_BUFFER_TOO_SMALL = -1016,
    GC_ERR_INVALID_INDEX = -1017,
    GC_ERR_PARSING_CHUNK_DATA = -1018,
    GC_ERR_INVALID_VALUE = -1019,
    GC_ERR_RESOURCE_EXHAUSTED = -1020,
    GC_ERR_OUT_OF_MEMORY = -1021,
    GC_ERR_BUSY = -1022,
    GC_ERR_AMBIGUOUS = -1023,
}

type GC_ERROR = i32;

impl<T> From<GenTlResult<T>> for GCErrorKind
where
    T: std::fmt::Debug,
{
    fn from(result: GenTlResult<T>) -> Self {
        use GCErrorKind::*;
        if result.is_ok() {
            return GC_ERR_SUCCESS;
        }

        let err = result.unwrap_err();
        match err {
            GenTlError::NotOpened => GC_ERR_NOT_INITIALIZED,
            GenTlError::AccessDenied => GC_ERR_ACCESS_DENIED,
            GenTlError::InvalidAddress => GC_ERR_INVALID_ADDRESS,
            GenTlError::InvalidValue(..) => GC_ERR_INVALID_VALUE,
            GenTlError::IoError(..) => GC_ERR_IO,
            GenTlError::ResourceInUse => GC_ERR_RESOURCE_IN_USE,
            GenTlError::InvalidId(..) => GC_ERR_INVALID_ID,
            GenTlError::InvalidIndex => GC_ERR_INVALID_INDEX,
            GenTlError::Timeout => GC_ERR_TIMEOUT,
        }
    }
}

struct LastError {
    code: GCErrorKind,
    text: Cow<'static, str>,
}

lazy_static::lazy_static! {
    static ref IS_MODULE_INITIALIZED: RwLock<bool> = RwLock::new(false);
}

thread_local! {
    static LAST_ERROR: RefCell<LastError> = {
        let last_error = LastError {
            code: GCErrorKind::GC_ERR_SUCCESS,
            text: "No Error".into(),
        };
        RefCell::new(last_error)
    }
}

macro_rules! assert_lib_initialized {
    () => {
        if !*IS_MODULE_INITIALIZED.read().unwrap() {
            let err = GCErrorKind::GC_ERR_NOT_INITIALIZED;
            save_last_error(err, "lib NOT initialized");
            return err as GC_ERROR;
        }
    };
}

macro_rules! try_gentl {
    ($expr:expr) => {{
        let res = $expr;
        match &res {
            Ok(x) => *x,
            Err(e) => {
                let text = format!("{}", e);
                let kind: GCErrorKind = res.into();
                save_last_error(kind, text);
                return kind as GC_ERROR;
            }
        }
    }};
}

fn save_last_error(kind: GCErrorKind, text: impl Into<Cow<'static, str>>) {
    LAST_ERROR.with(|err| {
        let mut err = err.borrow_mut();
        err.code = kind;
        err.text = text.into();
    })
}

fn copy_str(src: &str, dst: *mut libc::c_char) -> GenTlResult<libc::size_t> {
    if !src.is_ascii() {
        return Err(GenTlError::InvalidValue("string is not ascii".into()));
    }

    let len_without_null = src.len();
    unsafe {
        std::ptr::copy_nonoverlapping(src.as_ptr() as *const libc::c_char, dst, len_without_null);
        dst.offset(len_without_null as isize).write(0); // Null terminated.
    }

    Ok(len_without_null + 1)
}

#[no_mangle]
pub extern "C" fn GCInitLib() -> GC_ERROR {
    let mut is_init = IS_MODULE_INITIALIZED.write().unwrap();
    if *is_init {
        let code = GCErrorKind::GC_ERR_RESOURCE_IN_USE;
        save_last_error(
            code,
            "multiple calles to GCInitLib without calling GCCloseLib",
        );
        code as GC_ERROR
    } else {
        *is_init = true;
        GCErrorKind::GC_ERR_SUCCESS as GC_ERROR
    }
}

#[no_mangle]
pub extern "C" fn GCCloseLib() -> GC_ERROR {
    let mut is_init = IS_MODULE_INITIALIZED.write().unwrap();
    if !*is_init {
        let code = GCErrorKind::GC_ERR_NOT_INITIALIZED;
        save_last_error(code, "GCCloseLib is called without calling GCInitLib");
        code as GC_ERROR
    } else {
        *is_init = false;
        GCErrorKind::GC_ERR_SUCCESS as GC_ERROR
    }
}

#[no_mangle]
pub extern "C" fn CGCGetInfo(
    _iInfoCmd: i32,
    _piType: i32,
    _pBuffer: *mut libc::c_void,
    _piSize: *mut libc::size_t,
) -> GC_ERROR {
    assert_lib_initialized!();
    let code = GCErrorKind::GC_ERR_NOT_IMPLEMENTED;
    save_last_error(code, "GCGetInfo is not implemented, use TLGetInfo instead");
    code as GC_ERROR
}

#[no_mangle]
pub extern "C" fn GCGetLastError(
    piErrorCode: *mut GC_ERROR,
    sErrorText: *mut libc::c_char,
    piSize: *mut libc::size_t,
) -> GC_ERROR {
    LAST_ERROR.with(|err| unsafe {
        let err = err.borrow();
        if err.code != GCErrorKind::GC_ERR_SUCCESS {
            let size = try_gentl!(copy_str(&err.text, sErrorText));
            *piErrorCode = err.code as GC_ERROR;
            *piSize = size;
        }
        GCErrorKind::GC_ERR_SUCCESS as GC_ERROR
    })
}
