use uefi::proto::media::file::FileImpl;
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::Status;

pub extern "efiapi" fn open_volume(
    this: &mut SimpleFileSystem,
    root: &mut *mut FileImpl,
) -> Status {
    todo!()
}
