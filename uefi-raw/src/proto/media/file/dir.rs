use super::RegularFile;

/// A `FileHandle` that is also a directory.
///
/// Use `File::into_type` or `Directory::new` to create a `Directory`. In
/// addition to supporting the normal `File` operations, `Directory`
/// supports iterating over its contained files.
#[repr(transparent)]
#[derive(Debug)]
pub struct Directory(RegularFile);
