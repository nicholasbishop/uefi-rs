use super::FileHandle;

/// A `FileHandle` that is also a regular (data) file.
///
/// Use `FileHandle::into_type` or `RegularFile::new` to create a `RegularFile`.
/// In addition to supporting the normal `File` operations, `RegularFile`
/// supports direct reading and writing.
#[repr(transparent)]
#[derive(Debug)]
pub struct RegularFile(FileHandle);
