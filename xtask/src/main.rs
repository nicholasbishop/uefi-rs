mod cargo;
mod qemu;
mod util;

use cargo::{Cargo, CargoAction, Features, Package};
use clap::{Parser, Subcommand};
use util::{run_cmd, Result, UefiArch, Verbose};

/// Developer utility for running various tasks in uefi-rs.
#[derive(Debug, Parser)]
struct Opt {
    #[clap(long, default_value_t)]
    target: UefiArch,

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
    Build,
    Clippy,
    Doc {
        #[clap(long)]
        open: bool,
    },
    Run,
    Test,
}

fn build(opt: &Opt) -> Result<()> {
    let cargo = Cargo {
        action: CargoAction::Build,
        features: Features::MoreCode,
        nightly: true,
        packages: Package::all_except_xtask(),
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
        target: None,
        warnings_as_errors: opt.warnings_as_errors,
    };
    run_cmd(cargo.command()?, opt.verbose())
}

fn doc(opt: &Opt, open: bool) -> Result<()> {
    let cargo = Cargo {
        action: CargoAction::Doc { open },
        features: Features::MoreCode,
        nightly: true,
        packages: Package::published(),
        target: Some(opt.target),
        warnings_as_errors: opt.warnings_as_errors,
    };
    run_cmd(cargo.command()?, opt.verbose())
}

fn run(opt: &Opt) -> Result<()> {
    // Build uefi-test-runner.
    let cargo = Cargo {
        action: CargoAction::Build,
        features: Features::Qemu,
        nightly: true,
        packages: vec![Package::UefiTestRunner],
        target: Some(opt.target),
        warnings_as_errors: opt.warnings_as_errors,
    };
    run_cmd(cargo.command()?, opt.verbose())?;
    qemu::run_qemu(
        // TODO
        opt.target,
        opt.verbose(),
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

    match opt.action {
        Action::Build => build(opt),
        Action::Clippy => clippy(opt),
        Action::Doc { open } => doc(opt, open),
        Action::Run => run(opt),
        Action::Test => test(opt),
    }
}
