use crate::{guid, Char16, Guid, Handle, Status};
use core::ffi::c_void;

pub type HiiHandle = *mut c_void;

#[derive(Debug)]
#[repr(C)]
pub struct HiiPackageListHeader {
    pub package_list_guid: Guid,
    pub package_length: u32,
}

newtype_enum! {
    pub enum HiiDatabaseNotifyType: usize => {
        NEW_PACK = 1,
        REMOVE_PACK = 2,
        EXPORT_PACK = 4,
        ADD_PACK = 8,
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct HiiPackageHeader {
    pub length_and_type: u32,
    pub data: [u8; 0],
}

pub type HiiDatabaseNotify = unsafe extern "efiapi" fn(
    package_type: u8,
    package_guid: *const Guid,
    package: *const HiiPackageHeader,
    handle: HiiHandle,
    notify_type: HiiDatabaseNotifyType,
) -> Status;

#[derive(Debug)]
#[repr(C)]
pub struct HiiDatabaseProtocol {
    pub new_package_list: unsafe extern "efiapi" fn(
        this: *const Self,
        package_list: *const HiiPackageListHeader,
        driver_handle: Handle,
        handle: *mut HiiHandle,
    ) -> Status,
    pub remove_package_list:
        unsafe extern "efiapi" fn(this: *const Self, handle: HiiHandle) -> Status,
    pub update_package_list: unsafe extern "efiapi" fn(
        this: *const Self,
        handle: HiiHandle,
        package_list: *const HiiPackageListHeader,
    ) -> Status,
    pub list_package_lists: unsafe extern "efiapi" fn(
        this: *const Self,
        package_type: u8,
        package_guid: *const Guid,
        handle_buffer_length: usize,
        handle: *mut HiiHandle,
    ) -> Status,
    pub export_package_lists: unsafe extern "efiapi" fn(
        this: *const Self,
        handle: HiiHandle,
        buffer_size: *mut usize,
        buffer: *mut HiiPackageListHeader,
    ) -> Status,
    pub register_package_notify: unsafe extern "efiapi" fn(
        this: *const Self,
        package_type: u8,
        package_guid: *const Guid,
        package_notify_fn: HiiDatabaseNotify,
        notify_type: HiiDatabaseNotifyType,
        notify_handle: Handle,
    ) -> Status,
    pub unregister_package_notify:
        unsafe extern "efiapi" fn(this: *const Self, notification_handle: Handle) -> Status,
    pub find_keyboard_layouts: unsafe extern "efiapi" fn(
        this: *const Self,
        key_guid_buffer_length: *mut u16,
        key_guid_buffer: *mut Guid,
    ) -> Status,
    pub get_keyboard_layout: unsafe extern "efiapi" fn(
        this: *const Self,
        key_guid: *const Guid,
        keyboard_layout_length: *mut u16,
        keyboard_layout: *mut HiiKeyboardLayout,
    ) -> Status,
    pub set_keyboard_layout:
        unsafe extern "efiapi" fn(this: *const Self, key_guid: *const Guid) -> Status,
    pub get_package_list_handle: unsafe extern "efiapi" fn(
        this: *const Self,
        package_list_handle: HiiHandle,
        driver_handle: *mut Handle,
    ) -> Status,
}

impl HiiPackageListHeader {
    pub const GUID: Guid = guid!("ef9fc172-a1b2-4693-b327-6d32fc416042");
}

#[derive(Debug)]
#[repr(C)]
pub struct HiiKeyboardLayout {
    pub layout_length: u16,
    pub guid: Guid,
    pub layout_descriptor_string_offset: u32,
    pub descriptor_count: u8,
    pub descriptors: [KeyDescriptor; 0],
}

#[derive(Debug)]
#[repr(C)]
pub struct KeyDescriptor {
    pub key: Key,
    pub unicode: Char16,
    pub shifted_unicode: Char16,
    pub alt_gr_unicode: Char16,
    pub shifted_alt_gr_unicode: Char16,
    pub modifier: u16,
    pub affected_attribute: u16,
}

newtype_enum! {
    pub enum Key: u32 => {
        LEFT_CTRL = 0,
        A0 = 1,
        LEFT_ALT = 2,
        SPACE_BAR = 3,
        A2 = 4,
        A3 = 5,
        A4 = 6,
        RIGHT_CTRL = 7,
        LEFT_ARROW = 8,
        DOWN_ARROW = 9,
        RIGHT_ARROW = 10,
        ZERO = 11,
        PERIOD = 12,
        ENTER = 13,
        LEFT_SHIFT = 14,
        B0 = 15,
        B1 = 16,
        B2 = 17,
        B3 = 18,
        B4 = 19,
        B5 = 20,
        B6 = 21,
        B7 = 22,
        B8 = 23,
        B9 = 24,
        B10 = 25,
        RIGHT_SHIFT = 26,
        UP_ARROW = 27,
        ONE = 28,
        TWO = 29,
        THREE = 30,
        CAPSLOCK = 31,
        C1 = 32,
        C2 = 33,
        C3 = 34,
        C4 = 35,
        C5 = 36,
        C6 = 37,
        C7 = 38,
        C8 = 39,
        C9 = 40,
        C10 = 41,
        C11 = 42,
        C12 = 43,
        FOUR = 44,
        FIVE = 45,
        SIX = 46,
        PLUS = 47,
        TAB = 48,
        D1 = 49,
        D2 = 50,
        D3 = 51,
        D4 = 52,
        D5 = 53,
        D6 = 54,
        D7 = 55,
        D8 = 56,
        D9 = 57,
        D10 = 58,
        D11 = 59,
        D12 = 60,
        D13 = 61,
        DEL = 62,
        END = 63,
        PAGE_DOWN = 64,
        SEVEN = 65,
        EIGHT = 66,
        NINE = 67,
        E0 = 68,
        E1 = 69,
        E2 = 70,
        E3 = 71,
        E4 = 72,
        E5 = 73,
        E6 = 74,
        E7 = 75,
        E8 = 76,
        E9 = 77,
        E10 = 78,
        E11 = 79,
        E12 = 80,
        BACKSPACE = 81,
        INS = 82,
        HOME = 83,
        PAGE_UP = 84,
        NUMLOCK = 85,
        SLASH = 86,
        ASTERISK = 87,
        MINUS = 88,
        ESC = 89,
        F1 = 90,
        F2 = 91,
        F3 = 92,
        F4 = 93,
        F5 = 94,
        F6 = 95,
        F7 = 96,
        F8 = 97,
        F9 = 98,
        F10 = 99,
        F11 = 100,
        F12 = 101,
        PRINT = 102,
        SCREEN_LOCK = 103,
        PAUSE = 104,
        INTL0 = 105,
        INTL1 = 106,
        INTL2 = 107,
        INTL3 = 108,
        INTL4 = 109,
        INTL5 = 110,
        INTL6 = 111,
        INTL7 = 112,
        INTL8 = 113,
        INTL9 = 114,
    }
}
