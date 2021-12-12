mod cargo;

use anyhow::{bail, Error};
use cargo::{Cargo, CargoAction, Features, Packages, Triple};
use clap::{Parser, Subcommand};
use std::process::Command;

type Result<T> = std::result::Result<T, Error>;

/// Developer utility for running various tasks in uefi-rs.
#[derive(Debug, Parser)]
struct Opt {
    /// Print commands before executing them.
    #[clap(long)]
    verbose: bool,

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

fn run_cmd(mut cmd: Command, verbose: bool) -> Result<()> {
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
        packages: Packages::EverythingExceptXtask,
        target: Triple::X86_64UnknownUefi,
    };
    run_cmd(cargo.command(), opt.verbose)
}

fn clippy(opt: &Opt) -> Result<()> {
    // Run clippy on all the UEFI packages.
    let cargo = Cargo {
        action: CargoAction::Clippy,
        features: Features::MoreCode,
        nightly: true,
        packages: Packages::EverythingExceptXtask,
        target: Triple::X86_64UnknownUefi,
    };
    run_cmd(cargo.command(), opt.verbose)?;

    // Run clippy on xtask.
    let cargo = Cargo {
        action: CargoAction::Clippy,
        features: Features::None,
        nightly: false,
        packages: Packages::Xtask,
        target: Triple::Default,
    };
    run_cmd(cargo.command(), opt.verbose)
}

fn doc(opt: &Opt, open: bool) -> Result<()> {
    let cargo = Cargo {
        action: CargoAction::Doc { open },
        features: Features::MoreCode,
        nightly: true,
        packages: Packages::Published,
        target: Triple::X86_64UnknownUefi,
    };
    run_cmd(cargo.command(), opt.verbose)
}

fn main() -> Result<()> {
    let opt = &Opt::parse();

    match opt.action {
        Action::Build => build(opt),
        Action::Clippy => clippy(opt),
        Action::Doc { open } => doc(opt, open),
        _ => todo!(),
    }
}
