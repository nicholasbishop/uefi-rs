use anyhow::{anyhow, bail, Error};
use std::fmt;
use std::process::Command;
use std::str::FromStr;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy, Debug)]
pub enum Arch {
    AArch64UnknownUefi,
    X86_64UnknownUefi,
    // TODO
}

impl Arch {
    fn as_triple(self) -> &'static str {
        match self {
            Self::AArch64UnknownUefi => "aarch64-unknown-uefi",
            Self::X86_64UnknownUefi => "x86_64-unknown-uefi",
        }
    }
}

impl fmt::Display for Arch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_triple())
    }
}

impl FromStr for Arch {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "aarch64" | "aarch64-unknown-uefi" => Ok(Self::AArch64UnknownUefi),
            "x86_64" | "x86_64-unknown-uefi" => Ok(Self::X86_64UnknownUefi),
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
