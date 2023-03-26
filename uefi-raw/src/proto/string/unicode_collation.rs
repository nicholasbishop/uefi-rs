//! The Unicode Collation Protocol.
//!
//! This protocol is used in the boot services environment to perform
//! lexical comparison functions on Unicode strings for given languages.

use crate::{guid, Char16, Char8, Guid, Identify};

/// The Unicode Collation Protocol.
///
/// Used to perform case-insensitive comparisons of strings.
#[repr(C)]
pub struct UnicodeCollation {
    pub stri_coll: extern "efiapi" fn(this: &Self, s1: *const Char16, s2: *const Char16) -> isize,
    pub metai_match:
        extern "efiapi" fn(this: &Self, string: *const Char16, pattern: *const Char16) -> bool,
    pub str_lwr: extern "efiapi" fn(this: &Self, s: *mut Char16),
    pub str_upr: extern "efiapi" fn(this: &Self, s: *mut Char16),
    pub fat_to_str:
        extern "efiapi" fn(this: &Self, fat_size: usize, fat: *const Char8, s: *mut Char16),
    pub str_to_fat:
        extern "efiapi" fn(this: &Self, s: *const Char16, fat_size: usize, fat: *mut Char8) -> bool,
}

unsafe impl Identify for UnicodeCollation {
    const GUID: Guid = guid!("a4c751fc-23ae-4c3e-92e9-4964cf63f349");
}
