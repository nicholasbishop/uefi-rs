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
    Clippy(ActionClippy),
    Doc(ActionDoc),

    /// Run tests and doctests.
    Test,

    /// Run tests similar to what the CI checks.
    Presubmit,
}

/// Build the application packages.
#[derive(StructOpt)]
struct ActionBuild {
    /// UEFI target to build for
    #[structopt(default_value)]
    target: UefiTarget,

    /// build in release mode
    #[structopt(long)]
    release: bool,
}

/// Run clippy.
#[derive(StructOpt)]
struct ActionClippy {
    /// treat warnings as errors
    #[structopt(long)]
    treat_warnings_as_errors: bool,
}

/// Generate documentation.
#[derive(StructOpt)]
struct ActionDoc {
    /// open the docs in a browser
    #[structopt(long)]
    open: bool,
}

fn build(action: &ActionBuild) -> Result<(), Error> {
    run_cargo(CargoCommand::Build {
        packages: &[Package::Template, Package::UefiTestRunner],
        target: action.target,
        release: action.release,
        extra_args: &["-Zbuild-std=core,compiler_builtins,alloc"],
    })
}

fn clippy(action: &ActionClippy) -> Result<(), Error> {
    run_cargo(CargoCommand::Clippy {
        treat_warnings_as_errors: action.treat_warnings_as_errors,
        features: &["alloc", "exts", "logger"],
    })
}

fn doc(action: &ActionDoc) -> Result<(), Error> {
    run_cargo(CargoCommand::Doc {
        no_deps: true,
        packages: &[Package::Uefi, Package::UefiMacros, Package::UefiServices],
        features: &["alloc", "exts", "logger"],
        open: action.open,
    })
}

fn test() -> Result<(), Error> {
    run_cargo(CargoCommand::Test {
        workspace: true,
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

fn presubmit() -> Result<(), Error> {
    run_cargo(CargoCommand::Format { check: true })?;
    clippy(&ActionClippy {
        treat_warnings_as_errors: true,
    })?;
    test()
}

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();

    match &opt.action {
        Action::Build(action) => build(action),
        Action::Doc(action) => doc(action),
        Action::Clippy(action) => clippy(action),
        Action::Test => test(),
        Action::Presubmit => presubmit(),
    }
}
