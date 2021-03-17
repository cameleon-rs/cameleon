#![allow(
    clippy::module_name_repetitions,
    clippy::clippy::similar_names,
    clippy::clippy::missing_errors_doc
)]

// TODO: Remove #[allow(unused)]
#[allow(non_snake_case, non_camel_case_types, unused)]
pub mod ffi;

#[allow(unused)]
mod imp;

use thiserror::Error;

/// Errors defined in GenTL specification.
#[allow(dead_code)]
#[derive(Error, Debug)]
pub(crate) enum GenTlError {
    /// Unspecified runtime error.
    #[error("unspecified runtime error")]
    Error(String),

    /// Module or resource not initialized.
    #[error("module or resource not initialized")]
    NotInitialized,

    /// Requested operation not implemented.
    #[error("requested operation not implemented")]
    NotImplemented,

    /// Requested resource is already in use.
    #[error("requested resource is already in use")]
    ResourceInUse,

    /// The access to the requested register address is denied because the register is not writable
    /// or because the Port module is opened in a way that it does not allow write access.
    #[error("the access to the requested register addresss is denied")]
    AccessDenied,

    /// Given handle does not support the operation.
    #[error("given handle does not support the operation")]
    InvalidHandle,

    /// ID doesn't reference any module or remote device.
    #[error("given ID doesn't reference any module or remote device: {}", 0)]
    InvalidId(String),

    /// The function has no data to work on or the data does not provide reliable information corresponding with the request.
    #[error(
        "the function has no data to work on or the data does not provide reliable information"
    )]
    NoData,

    /// One of the parameter given was not valid or out of range.
    #[error("one of the parameter given was not valid or out of range")]
    InvalidParameter,

    /// Communication error or connection lost.
    #[error("communication error or connection lost: {}", 0)]
    Io(Box<dyn std::error::Error>),

    /// Operation timed out.
    #[error("operation timed out")]
    Timeout,

    /// An operation has been aborted before it could be completed.
    #[error("an operation has been aborted before it could be completed")]
    Abort,

    /// The GenTL Consumer has not announced enough buffers to start the acquisition.
    #[error("the GenTL Consumer has not announced enough buffers to start the acquisition")]
    InvalidBuffer,

    /// Resource or information is not available at a given time in a current state.
    #[error("resource or information is not available at a given time in a current state")]
    NotAvailable,

    /// There is no register with the provided address.
    #[error("there is no register with the provided address")]
    InvalidAddress,

    /// A provided buffer is too small to receive the expected amount of data.
    #[error("a provided buffer is too small to receive the expected amount of data")]
    BufferTooSmall,

    /// A provided index referencing a Producer internal object is out of bounds.
    #[error("given index is out of range")]
    InvalidIndex,

    /// An error occurred parsing a buffer containing chunk data.
    #[error("an error occurred parsing a buffer containing chunk data")]
    ParsingChunkData,

    /// An invalid value has been written.
    #[error("an invalid value has been written: {}", 0)]
    InvalidValue(std::borrow::Cow<'static, str>),

    /// A requested resource is exhausted.
    #[error("a requested resource is exhausted")]
    ResourceExhausted,

    /// The system and/or other hardware in the system (frame grabber) ran out of memory.
    #[error("the system and/or other hardware in the system (frame grabber) ran out of memory")]
    OutOfMemory,

    /// The required operation cannot be executed because the responsible module/entity is busy.
    #[error(
        "the required operation cannot be executed because the responsible module/entity is busy"
    )]
    Busy,

    /// The required operation cannot be executed unambiguously in given context.
    #[error("the required operation cannot be executed unambiguously in given")]
    Ambiguous,
}

pub(crate) type GenTlResult<T> = std::result::Result<T, GenTlError>;
