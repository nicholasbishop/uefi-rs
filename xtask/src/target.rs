use anyhow::{bail, Error};
use std::fmt;
use std::str::FromStr;

/// UEFI target to build for.
#[derive(Clone, Copy)]
pub enum UefiTarget {
    Aarch64,
    X86_64,
}

impl Default for UefiTarget {
    fn default() -> Self {
        Self::X86_64
    }
}

impl fmt::Display for UefiTarget {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::Aarch64 => write!(f, "aarch64"),
            Self::X86_64 => write!(f, "x86_64"),
        }
    }
}

impl FromStr for UefiTarget {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "aarch64" => Ok(Self::Aarch64),
            "x86_64" => Ok(Self::X86_64),
            _ => bail!("invalid target: {}", s),
        }
    }
}

impl UefiTarget {
    pub fn target_triple(&self) -> &'static str {
        match self {
            Self::Aarch64 => "aarch64-unknown-uefi",
            Self::X86_64 => "x86_64-unknown-uefi",
        }
    }
}
