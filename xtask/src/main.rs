mod cargo;
mod qemu;

use anyhow::{bail, Error};
use cargo::{Cargo, CargoAction, Features, Package};
use clap::{Parser, Subcommand};
use std::process::Command;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy, Debug)]
pub enum Triple {
    Default,
    X86_64UnknownUefi,
    // TODO
}

/// Developer utility for running various tasks in uefi-rs.
#[derive(Debug, Parser)]
struct Opt {
    /// Print commands before executing them.
    #[clap(long)]
    verbose: bool,

    /// Treat warnings as errors.
    #[clap(long)]
    warnings_as_errors: bool,

    #[clap(subcommand)]
    action: Action,
}

#[derive(Debug, Subcommand)]
enum Action {
    Build,
    Clippy,
    Doc {
        #[clap(long)]
        open: bool,
    },
    Run,
    Test,
}

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

fn build(opt: &Opt) -> Result<()> {
    let cargo = Cargo {
        action: CargoAction::Build,
        features: Features::MoreCode,
        nightly: true,
        packages: Package::all_except_xtask(),
        target: Triple::X86_64UnknownUefi,
        warnings_as_errors: opt.warnings_as_errors,
    };
    run_cmd(cargo.command(), opt.verbose)
}

fn clippy(opt: &Opt) -> Result<()> {
    // Run clippy on all the UEFI packages.
    let cargo = Cargo {
        action: CargoAction::Clippy,
        features: Features::MoreCode,
        nightly: true,
        packages: Package::all_except_xtask(),
        target: Triple::X86_64UnknownUefi,
        warnings_as_errors: opt.warnings_as_errors,
    };
    run_cmd(cargo.command(), opt.verbose)?;

    // Run clippy on xtask.
    let cargo = Cargo {
        action: CargoAction::Clippy,
        features: Features::None,
        nightly: false,
        packages: vec![Package::Xtask],
        target: Triple::Default,
        warnings_as_errors: opt.warnings_as_errors,
    };
    run_cmd(cargo.command(), opt.verbose)
}

fn doc(opt: &Opt, open: bool) -> Result<()> {
    let cargo = Cargo {
        action: CargoAction::Doc { open },
        features: Features::MoreCode,
        nightly: true,
        packages: Package::published(),
        target: Triple::X86_64UnknownUefi,
        warnings_as_errors: opt.warnings_as_errors,
    };
    run_cmd(cargo.command(), opt.verbose)
}

fn run(opt: &Opt) -> Result<()> {
    // Build uefi-test-runner.
    let cargo = Cargo {
        action: CargoAction::Build,
        features: Features::Qemu,
        nightly: true,
        packages: vec![Package::UefiTestRunner],
        target: Triple::X86_64UnknownUefi,
        warnings_as_errors: opt.warnings_as_errors,
    };
    run_cmd(cargo.command(), opt.verbose)?;
    qemu::run_qemu(
        // TODO
        Triple::X86_64UnknownUefi,
        opt.verbose,
    )
}

fn test(opt: &Opt) -> Result<()> {
    let cargo = Cargo {
        action: CargoAction::Test,
        features: Features::Exts,
        nightly: false,
        // Don't test uefi-services (or the packages that depend on it)
        // as it has lang items that conflict with `std`. The xtask
        // currently doesn't have any tests.
        packages: vec![Package::Uefi, Package::UefiMacros],
        target: Triple::Default,
        // cargo test doesn't currently have a flag to treat warnings as
        // errors.
        warnings_as_errors: false,
    };
    run_cmd(cargo.command(), opt.verbose)
}

fn main() -> Result<()> {
    let opt = &Opt::parse();

    match opt.action {
        Action::Build => build(opt),
        Action::Clippy => clippy(opt),
        Action::Doc { open } => doc(opt, open),
        Action::Run => run(opt),
        Action::Test => test(opt),
    }
}
