mod cargo;
mod qemu;
mod util;

use anyhow::Result;
use cargo::{Cargo, CargoAction, Features, Package};
use clap::{Parser, Subcommand};
use qemu::QemuOpt;
use std::path::Path;
use util::{run_cmd, UefiArch, Verbose};

/// Developer utility for running various tasks in uefi-rs.
#[derive(Debug, Parser)]
struct Opt {
    #[clap(long, default_value_t)]
    target: UefiArch,

    /// Build in release mode.
    #[clap(long)]
    release: bool,

    /// Print commands before executing them.
    #[clap(long)]
    verbose: bool,

    /// Treat warnings as errors.
    #[clap(long)]
    warnings_as_errors: bool,

    #[clap(subcommand)]
    action: Action,
}

impl Opt {
    fn verbose(&self) -> Verbose {
        if self.verbose {
            Verbose::Yes
        } else {
            Verbose::No
        }
    }
}

#[derive(Debug, Subcommand)]
enum Action {
    /// Build all the uefi packages.
    Build,

    /// Run clippy on all the packages.
    Clippy,

    /// Build the docs for the uefi packages.
    Doc {
        /// Opens the docs in a browser.
        #[clap(long)]
        open: bool,
    },

    /// Build uefi-test-runner and run it in QEMU.
    Run(QemuOpt),

    /// Run unit tests and doctests on the host.
    Test,
}

fn build(opt: &Opt) -> Result<()> {
    let cargo = Cargo {
        action: CargoAction::Build,
        features: Features::MoreCode,
        nightly: true,
        packages: Package::all_except_xtask(),
        release: opt.release,
        target: Some(opt.target),
        warnings_as_errors: opt.warnings_as_errors,
    };
    run_cmd(cargo.command()?, opt.verbose())
}

fn clippy(opt: &Opt) -> Result<()> {
    // Run clippy on all the UEFI packages.
    let cargo = Cargo {
        action: CargoAction::Clippy,
        features: Features::MoreCode,
        nightly: true,
        packages: Package::all_except_xtask(),
        release: opt.release,
        target: Some(opt.target),
        warnings_as_errors: opt.warnings_as_errors,
    };
    run_cmd(cargo.command()?, opt.verbose())?;

    // Run clippy on xtask.
    let cargo = Cargo {
        action: CargoAction::Clippy,
        features: Features::None,
        nightly: false,
        packages: vec![Package::Xtask],
        release: opt.release,
        target: None,
        warnings_as_errors: opt.warnings_as_errors,
    };
    run_cmd(cargo.command()?, opt.verbose())
}

/// Build docs.
fn doc(opt: &Opt, open: bool) -> Result<()> {
    let cargo = Cargo {
        action: CargoAction::Doc { open },
        features: Features::MoreCode,
        nightly: true,
        packages: Package::published(),
        release: opt.release,
        target: Some(opt.target),
        warnings_as_errors: opt.warnings_as_errors,
    };
    run_cmd(cargo.command()?, opt.verbose())
}

/// Build uefi-test-runner and run it in QEMU.
fn run_vm_tests(opt: &Opt, qemu_opt: &QemuOpt) -> Result<()> {
    // Build uefi-test-runner.
    let cargo = Cargo {
        action: CargoAction::Build,
        features: Features::Qemu,
        nightly: true,
        packages: vec![Package::UefiTestRunner],
        release: opt.release,
        target: Some(opt.target),
        warnings_as_errors: opt.warnings_as_errors,
    };
    run_cmd(cargo.command()?, opt.verbose())?;

    // Create an EFI boot directory to pass into QEMU.
    let build_mode = if opt.release { "release" } else { "debug" };
    let build_dir = Path::new("target")
        .join(opt.target.as_triple())
        .join(build_mode);
    let esp_dir = build_dir.join("esp");
    let boot_dir = esp_dir.join("EFI").join("Boot");
    let built_file = build_dir.join("uefi-test-runner.efi");
    let output_file = match opt.target {
        UefiArch::AArch64 => "BootAA64.efi",
        UefiArch::X86_64 => "BootX64.efi",
    };
    if !boot_dir.exists() {
        fs_err::create_dir_all(&boot_dir)?;
    }
    fs_err::copy(built_file, boot_dir.join(output_file))?;

    qemu::run_qemu(opt.target, qemu_opt, &esp_dir, opt.verbose())
}

/// Run unit tests and doctests on the host. Most of uefi-rs is tested
/// with VM tests, but a few things like macros and data types can be
/// tested with regular tests.
fn run_host_tests(opt: &Opt) -> Result<()> {
    let cargo = Cargo {
        action: CargoAction::Test,
        features: Features::Exts,
        nightly: false,
        // Don't test uefi-services (or the packages that depend on it)
        // as it has lang items that conflict with `std`. The xtask
        // currently doesn't have any tests.
        packages: vec![Package::Uefi, Package::UefiMacros],
        release: opt.release,
        // Use the host target so that tests can run without a VM.
        target: None,
        // cargo test doesn't currently have a flag to treat warnings as
        // errors.
        warnings_as_errors: false,
    };
    run_cmd(cargo.command()?, opt.verbose())
}

fn main() -> Result<()> {
    let opt = &Opt::parse();

    match &opt.action {
        Action::Build => build(opt),
        Action::Clippy => clippy(opt),
        Action::Doc { open } => doc(opt, *open),
        Action::Run(qemu_opt) => run_vm_tests(opt, qemu_opt),
        Action::Test => run_host_tests(opt),
    }
}
