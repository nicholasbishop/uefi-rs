///! Facilities for dealing with UEFI operation results.

/// The error type that we use, essentially a status code + optional additional data
mod error;
pub use self::error::Error;

/// Definition of UEFI's standard status codes
mod status;
pub use self::status::Status;

/// Return type of most UEFI functions. Both success and error payloads are optional.
///
/// Almost all UEFI operations provide a status code as an output which
/// indicates either success, a warning, or an error. This type alias maps
/// [`Status::SUCCESS`] to the `Ok` variant (with optional `Output` data), and
/// maps both warning and error statuses to the `Err` variant of type [`Error`],
/// which may carry optional inner `ErrData`.
///
/// Warnings are treated as errors by default because they generally indicate
/// an abnormal situation.
///
/// Some convenience methods are provided by the [`ResultExt`] trait.
pub type Result<Output = (), ErrData = ()> = core::result::Result<Output, Error<ErrData>>;
