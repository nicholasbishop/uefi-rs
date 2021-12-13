use anyhow::{anyhow, bail, Error};
use std::fmt;
use std::process::Command;
use std::str::FromStr;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy, Debug)]
pub enum UefiArch {
    AArch64,
    X86_64,
    // TODO
}

impl UefiArch {
    fn as_triple(self) -> &'static str {
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
        write!(f, "{}", self.as_triple())
    }
}

impl FromStr for UefiArch {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "aarch64" | "aarch64-unknown-uefi" => Ok(Self::AArch64),
            "x86_64" | "x86_64-unknown-uefi" => Ok(Self::X86_64),
            _ => Err(anyhow!("invalid triple: {}", s)),
        }
    }
}

/// Run a `Command` and check that it completes successfully. Optionally
/// print the command before running it.
pub fn run_cmd(mut cmd: Command, verbose: bool) -> Result<()> {
    if verbose {
        print!("{}", cmd.get_program().to_string_lossy());
        for arg in cmd.get_args() {
            print!(" {}", arg.to_string_lossy());
        }
        println!();
    }

    let status = cmd.status()?;
    if status.success() {
        Ok(())
    } else {
        bail!("command failed: {}", status);
    }
}
