//! NES errors
//!
//! All errors NES can produce

use thiserror::Error;

/// NES error type
///
/// All NES errors are encapsuled inside this error type
#[derive(Debug, Error)]
pub enum NesError {
    #[error("NES can't run without a cartidge!")]
    NoCartidgeInserted,

    #[error(
        "Address out of bounds, index is ${address:0>4X} but memory size is ${memory_size:0>4X}"
    )]
    MemoryAccessError { address: u16, memory_size: usize },

    #[error("Bus error: {details}")]
    BusError {
        details: String,
        #[source]
        source: BusError,
    },

    #[error("NES UI error: {details}")]
    UiError {
        details: String,
        #[source]
        source: UiError,
    },

    #[error("NES internal error: {0}")]
    NesInternalError(String),
}

/// Bus errors
#[derive(Debug, Error)]
pub enum BusError {
    #[error("Device {device_id} already attached to bus {bus_id}")]
    AlreadyAttached {
        bus_id: &'static str,
        device_id: &'static str,
    },

    #[error("Bus '{bus_id}' doesn't have an attached device for address ${address:0>4X}")]
    MissingBusDevice { bus_id: String, address: u16 },

    #[error("Bus '{bus_id}' failed while reading from device '{device_id}' on address ${address:0>4X}: {details}")]
    BusReadError {
        bus_id: &'static str,
        device_id: &'static str,
        address: u16,
        details: String,
    },

    #[error("Bus '{bus_id}' failed while writting to device '{device_id}' on address ${address:0>4X}: {details}")]
    BusWriteError {
        bus_id: &'static str,
        device_id: &'static str,
        address: u16,
        details: String,
    },
}

/// UI errors
#[derive(Debug, Error)]
pub enum UiError {
    #[error("UI already started: {0}")]
    AlreadyStarted(String),

    #[error("UI is not started yet, consider starting it before doing this action")]
    NotStarted,

    #[error("unhandled UI error: {0}")]
    Unhandled(String),
}
