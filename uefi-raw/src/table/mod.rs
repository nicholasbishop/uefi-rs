//! Standard UEFI tables.

mod header;
pub use self::header::Header;

mod revision;
pub use self::revision::Revision;

mod system;
pub use self::system::SystemTable;

pub mod boot;
pub mod runtime;

pub mod cfg;
