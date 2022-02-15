///! Facilities for dealing with UEFI operation results.
///!
///! Almost all UEFI operations provide a status code as an output, which may
///! either indicate success, fatal failure, or non-fatal failure. In addition,
///! they may produce output, both in case of success and failure.
///!
///! We model this using an extended version of Rust's standard Result type,
///! whose successful path supports UEFI warnings and whose failing path can
///! report both an UEFI status code and extra data about the error.
///!
///! Convenience methods are also provided via extension traits to ease working
///! with this complex type in everyday usage.
use core::fmt::Debug;

/// The error type that we use, essentially a status code + optional additional data
mod error;
pub use self::error::Error;

/// Definition of UEFI's standard status codes
mod status;
pub use self::status::Status;

/// Return type of most UEFI functions. Both success and error payloads are optional.
pub type Result<Output = (), ErrData = ()> = core::result::Result<Output, Error<ErrData>>;

/// Extension trait for Result which helps dealing with UEFI's warnings
pub trait ResultExt<Output, ErrData: Debug> {
    /// Extract the UEFI status from this result
    fn status(&self) -> Status;

    /// Transform the ErrData value to ()
    fn discard_errdata(self) -> Result<Output>;
}

impl<Output, ErrData: Debug> ResultExt<Output, ErrData> for Result<Output, ErrData> {
    fn status(&self) -> Status {
        match self {
            Ok(_) => Status::SUCCESS,
            Err(e) => e.status(),
        }
    }

    fn discard_errdata(self) -> Result<Output> {
        match self {
            Ok(o) => Ok(o),
            Err(e) => Err(e.status().into()),
        }
    }
}
