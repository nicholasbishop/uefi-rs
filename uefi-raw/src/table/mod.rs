//! Standard UEFI tables.

pub mod boot;
pub mod cfg;
pub mod runtime;

mod header;
mod revision;
mod system;

pub use self::header::Header;
pub use self::revision::Revision;
pub use self::system::SystemTable;
