mod cargo;

use anyhow::Error;
use cargo::{run_cargo, CargoCommand, Package, UefiTarget};
use structopt::StructOpt;

/// Task runner for uefi.
#[derive(StructOpt)]
struct Opt {
    #[structopt(subcommand)]
    action: Action,
}

#[derive(Clone, Copy, StructOpt)]
enum CiJob {
    /// Run all of the jobs.
    All,
    /// Check formatting, run clippy, and check for docstring warnings.
    Lint,
    /// Run tests and documentation tests.
    Test,
    /// Build for the aarch64 UEFI target.
    BuildAarch64,
    /// Build and run tests on the x86_64 UEFI target.
    BuildAndTestX86_64,
}

#[derive(StructOpt)]
enum Action {
    Build(ActionBuild),
    Clippy(ActionClippy),
    Doc(ActionDoc),

    /// Run CI checks.
    Ci {
        #[structopt(subcommand)]
        job: CiJob,
    },

    /// Run tests and doctests.
    Test,
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

    /// treat warnings as errors
    #[structopt(long)]
    treat_warnings_as_errors: bool,
}

fn build(action: ActionBuild) -> Result<(), Error> {
    run_cargo(CargoCommand::Build {
        packages: &Package::apps(),
        target: action.target,
        release: action.release,
        extra_args: &["-Zbuild-std=core,compiler_builtins,alloc"],
    })
}

fn ci(job: CiJob) -> Result<(), Error> {
    match job {
        CiJob::All => {
            ci(CiJob::Lint)?;
            ci(CiJob::Test)?;
            ci(CiJob::BuildAarch64)?;
            ci(CiJob::BuildAndTestX86_64)?;
        }
        CiJob::Lint => {
            run_cargo(CargoCommand::Format { check: true })?;
            clippy(ActionClippy {
                treat_warnings_as_errors: true,
            })?;
            doc(ActionDoc {
                open: false,
                treat_warnings_as_errors: true,
            })?;
        }
        CiJob::Test => {
            test()?;
        }
        CiJob::BuildAarch64 => {
            build(ActionBuild {
                target: UefiTarget::Aarch64,
                release: false,
            })?;
        }
        CiJob::BuildAndTestX86_64 => todo!(),
    }
    Ok(())
}

fn clippy(action: ActionClippy) -> Result<(), Error> {
    run_cargo(CargoCommand::Clippy {
        treat_warnings_as_errors: action.treat_warnings_as_errors,
        features: &["alloc", "exts", "logger"],
    })
}

fn doc(action: ActionDoc) -> Result<(), Error> {
    run_cargo(CargoCommand::Doc {
        no_deps: true,
        packages: &Package::libraries(),
        features: &["alloc", "exts", "logger"],
        open: action.open,
        treat_warnings_as_errors: action.treat_warnings_as_errors,
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

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();

    match opt.action {
        Action::Build(action) => build(action),
        Action::Ci { job } => ci(job),
        Action::Clippy(action) => clippy(action),
        Action::Doc(action) => doc(action),
        Action::Test => test(),
    }
}
