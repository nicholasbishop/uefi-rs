use crate::uefi_stub::boot::install_protocol_simple;
use std::ptr;
use uefi::proto::device_path::media::{PartitionFormat, PartitionSignature};
use uefi::proto::device_path::{DevicePath, DevicePathNode, DevicePathNodeEnum};
use uefi::{CString16, Result};
use uefi_raw::protocol::device_path::{
    DevicePathFromTextProtocol, DevicePathProtocol, DevicePathToTextProtocol,
};
use uefi_raw::table::boot::MemoryType;
use uefi_raw::Char16;

fn device_node_to_string(node: &DevicePathNode) -> Option<String> {
    let Ok(node) = node.as_enum() else {
        return None;
    };
    match node {
        DevicePathNodeEnum::AcpiAcpi(node) => {
            if node.hid() == 0x41d0_0a03 {
                Some("PciRoot(0x0)".into())
            } else {
                None
            }
        }
        DevicePathNodeEnum::HardwarePci(node) => {
            Some(format!("Pci({:#X},{:#X})", node.device(), node.function()))
        }
        DevicePathNodeEnum::MediaHardDrive(node) => {
            let partition_format = match node.partition_format() {
                PartitionFormat::MBR => "MBR".to_string(),
                PartitionFormat::GPT => "GPT".to_string(),
                n => format!("{:#X}", n.0),
            };
            let partition_signature = match node.partition_signature() {
                PartitionSignature::Mbr(n) => format!("{:#X}", u32::from_le_bytes(n)),
                PartitionSignature::Guid(g) => g.to_string(),
                _ => return None,
            };
            Some(format!(
                "HD({},{partition_format},{partition_signature},{:#X},{:#X})",
                node.partition_number(),
                node.partition_start(),
                node.partition_size()
            ))
        }
        DevicePathNodeEnum::MediaFilePath(node) => {
            Some(node.path_name().to_cstring16().unwrap().to_string())
        }
        DevicePathNodeEnum::MessagingSata(node) => Some(format!(
            "Sata({:#X},{:#X},{:#X})",
            node.hba_port_number(),
            node.port_multiplier_port_number(),
            node.logical_unit_number()
        )),
        _ => None,
    }
}

fn str_to_cstring16_pool_alloc(s: &str) -> *const Char16 {
    let s = CString16::try_from(s).unwrap();
    let mut alloc: *mut u8 = ptr::null_mut();
    if !crate::uefi_stub::boot::allocate_pool(
        MemoryType::BOOT_SERVICES_DATA,
        s.num_bytes(),
        &mut alloc,
    )
    .is_success()
    {
        return ptr::null_mut();
    }
    unsafe {
        alloc.copy_from(s.as_ptr().cast(), s.num_bytes());
    }
    alloc.cast()
}

pub extern "efiapi" fn convert_device_node_to_text(
    device_node: *const DevicePathProtocol,
    display_only: bool,
    allow_shortcuts: bool,
) -> *const Char16 {
    let node = unsafe { DevicePathNode::from_ffi_ptr(device_node.cast()) };
    let Some(s) = device_node_to_string(node) else {
        return ptr::null();
    };
    str_to_cstring16_pool_alloc(&s)
}

pub extern "efiapi" fn convert_device_path_to_text(
    device_path: *const DevicePathProtocol,
    display_only: bool,
    allow_shortcuts: bool,
) -> *const Char16 {
    let device_path = unsafe { DevicePath::from_ffi_ptr(device_path.cast()) };
    let s = device_path
        .node_iter()
        .map(|n| device_node_to_string(n).unwrap())
        .collect::<Vec<_>>()
        .join("/");
    str_to_cstring16_pool_alloc(&s)
}

pub extern "efiapi" fn convert_text_to_device_node(
    text_device_node: *const Char16,
) -> *const DevicePathProtocol {
    // TODO
    ptr::null()
}

pub extern "efiapi" fn convert_text_to_device_path(
    text_device_path: *const Char16,
) -> *const DevicePathProtocol {
    // TODO
    ptr::null()
}

pub static DEVICE_PATH_TO_TEXT_INTERFACE: DevicePathToTextProtocol = DevicePathToTextProtocol {
    convert_device_node_to_text,
    convert_device_path_to_text,
};

pub static DEVICE_PATH_FROM_TEXT_INTERFACE: DevicePathFromTextProtocol =
    DevicePathFromTextProtocol {
        convert_text_to_device_node,
        convert_text_to_device_path,
    };

pub fn install() -> Result {
    let ptr: *const _ = &DEVICE_PATH_TO_TEXT_INTERFACE;
    install_protocol_simple(None, &DevicePathToTextProtocol::GUID, ptr.cast())?;

    let ptr: *const _ = &DEVICE_PATH_FROM_TEXT_INTERFACE;
    install_protocol_simple(None, &DevicePathFromTextProtocol::GUID, ptr.cast())?;

    Ok(())
}
