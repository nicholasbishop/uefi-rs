mod cargo;
mod qemu;

use anyhow::Error;
use cargo::{run_cargo, CargoCommand, Package};
use qemu::run_qemu_test;
use structopt::StructOpt;

/// UEFI target to build for.
#[derive(Clone, Copy, Eq, PartialEq, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum UefiTarget {
    Aarch64,
    X86_64,
}

impl Default for UefiTarget {
    fn default() -> Self {
        Self::X86_64
    }
}

impl UefiTarget {
    pub fn target_triple(&self) -> &str {
        match self {
            Self::Aarch64 => "aarch64-unknown-uefi",
            Self::X86_64 => "x86_64-unknown-uefi",
        }
    }
}

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
    Qemu(ActionQemu),

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

/// Build uefi-test-runner and run under QEMU.
#[derive(StructOpt)]
pub struct ActionQemu {
    /// UEFI target to build for
    #[structopt(default_value)]
    target: UefiTarget,

    /// build in release mode
    #[structopt(long)]
    release: bool,

    /// run without a GUI
    #[structopt(long)]
    headless: bool,

    /// run without KVM acceleration
    #[structopt(long)]
    no_kvm: bool,

    /// exit instead of reboot
    #[structopt(long)]
    no_reboot: bool,
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
        CiJob::BuildAndTestX86_64 => {
            todo!();
        }
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

fn qemu(action: ActionQemu) -> Result<(), Error> {
    build(ActionBuild {
        target: action.target,
        release: action.release,
    })?;

    run_qemu_test(action)
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
        Action::Qemu(action) => qemu(action),
        Action::Test => test(),
    }
}
