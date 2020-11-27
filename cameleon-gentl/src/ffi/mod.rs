#[macro_use]
mod macros;

pub mod interface;
pub mod system;

use std::{cell::RefCell, mem::ManuallyDrop, sync::RwLock};

use crate::{GenTlError, GenTlResult};

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct GC_ERROR(i32);

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct bool8_t(u8);

impl Into<bool> for bool8_t {
    fn into(self) -> bool {
        self.0 != 0
    }
}

impl Into<GC_ERROR> for &GenTlError {
    fn into(self) -> GC_ERROR {
        use GenTlError::*;
        let code = match self {
            Error(..) => -1001,
            NotInitialized => -1002,
            NotImplemented => -1003,
            ResourceInUse => -1004,
            AccessDenied => -1005,
            InvalidModuleHandle => -1006,
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

enum ModuleHandle<'a> {
    System(&'a system::SystemModule),
    Interface(&'a interface::InterfaceModule),
}

impl<'a> ModuleHandle<'a> {
    fn system(&self) -> GenTlResult<&'a system::SystemModule> {
        match self {
            ModuleHandle::System(system) => Ok(system),
            _ => Err(GenTlError::InvalidHandle),
        }
    }

    unsafe fn from_raw_manually_drop(
        raw_handle: *mut libc::c_void,
    ) -> GenTlResult<ManuallyDrop<Box<ModuleHandle<'a>>>> {
        if !raw_handle.is_null() {
            let handle = raw_handle as *mut ModuleHandle;
            Ok(ManuallyDrop::new(Box::from_raw(handle)))
        } else {
            Err(GenTlError::InvalidHandle)
        }
    }

    unsafe fn into_raw(self: Box<Self>) -> *mut libc::c_void {
        Box::into_raw(self) as *mut libc::c_void
    }
}

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
    static ref IS_LIB_INITIALIZED: RwLock<bool> = RwLock::new(false);
}

thread_local! {
    static LAST_ERROR: RefCell<LastError> = {
        let last_error = LastError {
            err: None,
        };
        RefCell::new(last_error)
    }
}

impl crate::imp::port::TlType {
    fn as_str(self) -> &'static str {
        use super::imp::port::TlType::*;
        match self {
            CameraLink => "CL",
            CameraLinkHS => "CLHS",
            CoaXPress => "CXP",
            GigEVision => "GEV",
            USB3Vision => "U3V",
            Mixed => "Mixed",
        }
    }
}

impl crate::imp::CharEncoding {
    fn as_raw(self) -> i32 {
        match self {
            Self::Ascii => 0,
            Self::UTF8 => 1,
        }
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

fn copy_info<T: CopyTo>(
    src: T,
    dst: *mut T::Destination,
) -> GenTlResult<(libc::size_t, INFO_DATATYPE)> {
    src.copy_to(dst).map(|size| (size, T::info_data_type()))
}

trait CopyTo {
    type Destination;

    fn copy_to(&self, dst: *mut Self::Destination) -> GenTlResult<libc::size_t>;

    fn info_data_type() -> INFO_DATATYPE;
}

impl CopyTo for &str {
    type Destination = libc::c_char;

    fn copy_to(&self, dst: *mut Self::Destination) -> GenTlResult<libc::size_t> {
        if !self.is_ascii() {
            return Err(GenTlError::InvalidValue("string is not ascii".into()));
        }

        let len_without_null = self.len();
        if !dst.is_null() {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    self.as_ptr() as *const libc::c_char,
                    dst,
                    len_without_null,
                );
                dst.offset(len_without_null as isize).write(0); // Null terminated.
            }
        }

        Ok(len_without_null + 1)
    }

    fn info_data_type() -> INFO_DATATYPE {
        INFO_DATATYPE::INFO_DATATYPE_STRING
    }
}

macro_rules! impl_copy_to_for_numeric {
    ($ty:ty, $info_data_type:expr) => {
        impl CopyTo for $ty {
            type Destination = $ty;

            fn copy_to(&self, dst: *mut Self::Destination) -> GenTlResult<libc::size_t> {
                let len = std::mem::size_of::<$ty>();

                if !dst.is_null() {
                    unsafe {
                        *dst = *self;
                    }
                }
                Ok(len)
            }

            fn info_data_type() -> INFO_DATATYPE {
                $info_data_type
            }
        }
    };
}

impl_copy_to_for_numeric!(i16, INFO_DATATYPE::INFO_DATATYPE_INT16);
impl_copy_to_for_numeric!(u16, INFO_DATATYPE::INFO_DATATYPE_UINT16);
impl_copy_to_for_numeric!(i32, INFO_DATATYPE::INFO_DATATYPE_INT32);
impl_copy_to_for_numeric!(u32, INFO_DATATYPE::INFO_DATATYPE_UINT32);
impl_copy_to_for_numeric!(i64, INFO_DATATYPE::INFO_DATATYPE_INT64);
impl_copy_to_for_numeric!(u64, INFO_DATATYPE::INFO_DATATYPE_UINT64);

fn assert_lib_initialized() -> GenTlResult<()> {
    if *IS_LIB_INITIALIZED.read().unwrap() {
        Ok(())
    } else {
        Err(GenTlError::NotInitialized)
    }
}

gentl_api!(
    no_assert pub fn GCInitLib() -> GenTlResult<()> {
        let mut is_init = IS_LIB_INITIALIZED.write().unwrap();

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
        let mut is_init = IS_LIB_INITIALIZED.write().unwrap();
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
                let size = text.as_str().copy_to(sErrorText)?;
                (code, size)
            }
            None => {
                let size = "No Error".copy_to(sErrorText)?;
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
