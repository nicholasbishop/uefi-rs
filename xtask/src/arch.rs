use anyhow::{anyhow, Error, Result};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug)]
pub enum UefiArch {
    AArch64,
    X86_64,
}

impl UefiArch {
    fn all() -> &'static [Self] {
        &[Self::AArch64, Self::X86_64]
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::AArch64 => "aarch64",
            Self::X86_64 => "x86_64",
        }
    }

    pub fn as_triple(self) -> &'static str {
        match self {
            Self::AArch64 => "aarch64-unknown-uefi",
            Self::X86_64 => "x86_64-unknown-uefi",
        }
    }
}

impl Default for UefiArch {
    fn default() -> Self {
        Self::X86_64
    }
}

impl fmt::Display for UefiArch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for UefiArch {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::all()
            .iter()
            .find(|arch| arch.as_str() == s)
            .cloned()
            .ok_or_else(|| anyhow!("invalid triple: {}", s))
    }
}
