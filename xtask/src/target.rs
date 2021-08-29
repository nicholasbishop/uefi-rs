/// UEFI target to build for.
#[derive(Clone, Copy, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum UefiTarget {
    Aarch64,
    X86_64,
}

impl Default for UefiTarget {
    fn default() -> Self {
        Self::X86_64
    }
}

impl UefiTarget {
    pub fn target_triple(&self) -> &str {
        match self {
            Self::Aarch64 => "aarch64-unknown-uefi",
            Self::X86_64 => "x86_64-unknown-uefi",
        }
    }
}
