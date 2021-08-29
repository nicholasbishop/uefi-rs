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

    /// Run clippy
    Clippy,

    /// Run tests and doctests
    Test,
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

fn clippy() -> Result<(), Error> {
    todo!();
}

fn test() -> Result<(), Error> {
    run_cargo(CargoCommand::Test {
        exclude: &[
            // Exclude uefi-services for now as compilation fails with
            // duplicate lang item errors.
            Package::UefiServices,
            // Also exclude the two applications, as they both depend on
            // uefi-services.
            Package::Template,
            Package::UefiTestRunner,
        ],
        features: &["exts"],
    })
}

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();

    match &opt.action {
        Action::Build(action) => build(action),
        Action::Clippy => clippy(),
        Action::Test => test(),
    }
}
