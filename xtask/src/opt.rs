use crate::arch::UefiArch;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Developer utility for running various tasks in uefi-rs.
#[derive(Debug, Parser)]
pub struct Opt {
    #[clap(subcommand)]
    pub action: Action,
}

#[derive(Debug, Subcommand)]
pub enum Action {
    Build(BuildOpt),
    Clippy(ClippyOpt),
    Doc(DocOpt),
    Run(QemuOpt),

    /// Run unit tests and doctests on the host.
    Test,

    #[clap(subcommand)]
    Ci(CiJob),
}

/// Build all the uefi packages.
#[derive(Debug, Parser)]
pub struct BuildOpt {
    /// UEFI target to build for.
    #[clap(long, default_value_t)]
    pub target: UefiArch,

    /// Build in release mode.
    #[clap(long)]
    pub release: bool,
}

/// Run clippy on all the packages.
#[derive(Debug, Parser)]
pub struct ClippyOpt {
    /// UEFI target to build for.
    #[clap(long, default_value_t)]
    pub target: UefiArch,

    /// Treat warnings as errors.
    #[clap(long)]
    pub warnings_as_errors: bool,
}

/// Build the docs for the uefi packages.
#[derive(Debug, Parser)]
pub struct DocOpt {
    /// Open the docs in a browser.
    #[clap(long)]
    pub open: bool,

    /// Treat warnings as errors.
    #[clap(long)]
    pub warnings_as_errors: bool,
}

/// Build uefi-test-runner and run it in QEMU.
#[derive(Debug, Parser)]
pub struct QemuOpt {
    /// UEFI target to build for.
    #[clap(long, default_value_t)]
    pub target: UefiArch,

    /// Build in release mode.
    #[clap(long)]
    pub release: bool,

    /// Disable hardware accelerated virtualization support in QEMU.
    #[clap(long)]
    pub disable_kvm: bool,

    /// Disable some tests that don't work in the CI.
    #[clap(long)]
    pub ci: bool,

    /// Run QEMU without a GUI.
    #[clap(long)]
    pub headless: bool,

    /// Directory in which to look for OVMF files.
    #[clap(long)]
    pub ovmf_dir: Option<PathBuf>,
}

/// Run one (or all) of the CI jobs.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Subcommand)]
pub enum CiJob {
    #[clap(name = "build-aarch64")]
    BuildAArch64,
    #[clap(name = "vm-test-x86_64")]
    VmTestX86_64,
    HostTest,
    Lint,

    All,
}
