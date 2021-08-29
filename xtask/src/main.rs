mod cargo;
mod target;

use anyhow::Error;
use cargo::{run_cargo, CargoCommand, Package};
use structopt::StructOpt;
use target::UefiTarget;

/// Task runner for uefi.
#[derive(StructOpt)]
struct Opt {
    #[structopt(subcommand)]
    action: Action,
}

#[derive(StructOpt)]
enum Action {
    Build(ActionBuild),
}

/// Build all packages.
#[derive(StructOpt)]
struct ActionBuild {
    /// UEFI target to build for
    #[structopt(default_value)]
    target: UefiTarget,

    /// build in release mode
    #[structopt(long)]
    release: bool,
}

fn build(action: &ActionBuild) -> Result<(), Error> {
    for package in Package::apps() {
        run_cargo(CargoCommand::Build {
            package,
            target: action.target,
            release: action.release,
        })?;
    }

    Ok(())
}

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();

    match &opt.action {
        Action::Build(action) => build(action),
    }
}
