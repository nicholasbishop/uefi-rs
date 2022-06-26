use super::Status;
use core::fmt::Debug;

/// TODO
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// The string contained characters that could not be rendered and were skipped.
    WarnUnknownGlyph,
    /// The handle was closed, but the file was not deleted.
    WarnDeleteFailure,
    /// The handle was closed, but the data to the file was not flushed properly.
    WarnWriteFailure,
    /// The resulting buffer was too small, and the data was truncated.
    WarnBufferTooSmall,
    /// The data has not been updated within the timeframe set by local policy.
    WarnStaleData,
    /// The resulting buffer contains UEFI-compliant file system.
    WarnFileSystem,
    /// The operation will be processed across a system reset.
    WarnResetRequired,

    /// The image failed to load.
    LoadError,
    /// A parameter was incorrect.
    InvalidParameter,
    /// The operation is not supported.
    Unsupported,
    /// The buffer was not the proper size for the request.
    BadBufferSize,
    /// The buffer is not large enough to hold the requested data.
    /// The required buffer size is returned in the appropriate parameter.
    BufferTooSmall(usize),
    /// There is no data pending upon return.
    NotReady,
    /// The physical device reported an error while attempting the operation.
    DeviceError,
    /// The device cannot be written to.
    WriteProtected,
    /// A resource has run out.
    OutOfResources,
    /// An inconstency was detected on the file system.
    VolumeCorrupted,
    /// There is no more space on the file system.
    VolumeFull,
    /// The device does not contain any medium to perform the operation.
    NoMedia,
    /// The medium in the device has changed since the last access.
    MediaChanged,
    /// The item was not found.
    NotFound,
    /// Access was denied.
    AccessDenied,
    /// The server was not found or did not respond to the request.
    NoResponse,
    /// A mapping to a device does not exist.
    NoMapping,
    /// The timeout time expired.
    Timeout,
    /// The protocol has not been started.
    NotStarted,
    /// The protocol has already been started.
    AlreadyStarted,
    /// The operation was aborted.
    Aborted,
    /// An ICMP error occurred during the network operation.
    IcmpError,
    /// A TFTP error occurred during the network operation.
    TftpError,
    /// A protocol error occurred during the network operation.
    ProtocolError,
    /// The function encountered an internal version that was
    /// incompatible with a version requested by the caller.
    IncompatibleVersion,
    /// The function was not performed due to a security violation.
    SecurityViolation,
    /// A CRC error was detected.
    CrcError,
    /// Beginning or end of media was reached
    EndOfMedia,
    /// The end of the file was reached.
    EndOfFile,
    /// The language specified was invalid.
    InvalidLanguage,
    /// The security status of the data is unknown or compromised and
    /// the data must be updated or replaced to restore a valid security status.
    CompromisedData,
    /// There is an address conflict address allocation
    IpAddressConflict,
    /// A HTTP error occurred during the network operation.
    HttpError,
}

impl Error {
    /// Get error `Status`.
    pub fn status(&self) -> Status {
        todo!()
    }
}

// TODO
impl From<Status> for Error {
    fn from(_status: Status) -> Error {
        todo!();
    }
}
