//! UEFI character handling
//!
//! UEFI uses both Latin-1 and UCS-2 character encoding, this module implements
//! support for the associated character types.

/// A Latin-1 character
pub type Char8 = u8;

/// An UCS-2 code point
pub type Char16 = u16;
