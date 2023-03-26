//! The Unicode Collation Protocol.
//!
//! This protocol is used in the boot services environment to perform
//! lexical comparison functions on Unicode strings for given languages.

use crate::proto::unsafe_protocol;
use crate::{Char16, Char8};

/// The Unicode Collation Protocol.
///
/// Used to perform case-insensitive comparisons of strings.
#[repr(C)]
#[unsafe_protocol("a4c751fc-23ae-4c3e-92e9-4964cf63f349")]
pub struct UnicodeCollation {
    stri_coll: extern "efiapi" fn(this: &Self, s1: *const Char16, s2: *const Char16) -> isize,
    metai_match:
        extern "efiapi" fn(this: &Self, string: *const Char16, pattern: *const Char16) -> bool,
    str_lwr: extern "efiapi" fn(this: &Self, s: *mut Char16),
    str_upr: extern "efiapi" fn(this: &Self, s: *mut Char16),
    fat_to_str: extern "efiapi" fn(this: &Self, fat_size: usize, fat: *const Char8, s: *mut Char16),
    str_to_fat:
        extern "efiapi" fn(this: &Self, s: *const Char16, fat_size: usize, fat: *mut Char8) -> bool,
}
