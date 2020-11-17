use super::{GenTlError, GenTlResult};

#[allow(non_camel_case_types)]
#[allow(unused)]
#[repr(i32)]
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

    GC_ERR_CUSTOM_ID = -10000,
}

impl From<GenTlResult<()>> for GCErrorKind {
    fn from(result: GenTlResult<()>) -> Self {
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
