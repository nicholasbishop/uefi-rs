mod arch;
mod cargo;
mod opt;
mod qemu;
mod util;

use anyhow::Result;
use arch::UefiArch;
use cargo::{Cargo, CargoAction, Feature, Package};
use clap::Parser;
use opt::{Action, BuildOpt, CiJob, ClippyOpt, DocOpt, Opt, QemuOpt};
use std::path::Path;
use std::process::Command;
use util::{is_ci, run_cmd};

fn build(opt: &BuildOpt) -> Result<()> {
    let cargo = Cargo {
        action: CargoAction::Build,
        features: Feature::more_code(),
        nightly: true,
        packages: Package::all_except_xtask(),
        release: opt.release,
        target: Some(opt.target),
        warnings_as_errors: false,
    };
    run_cmd(cargo.command()?)
}

fn check_formatting() -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.args(&["fmt", "--check", "--all"]);
    run_cmd(cmd)
}

fn clippy(opt: &ClippyOpt) -> Result<()> {
    // Run clippy on all the UEFI packages.
    let cargo = Cargo {
        action: CargoAction::Clippy,
        features: Feature::more_code(),
        nightly: true,
        packages: Package::all_except_xtask(),
        release: false,
        target: Some(opt.target),
        warnings_as_errors: opt.warnings_as_errors,
    };
    run_cmd(cargo.command()?)?;

    // Run clippy on xtask.
    let cargo = Cargo {
        action: CargoAction::Clippy,
        features: Vec::new(),
        nightly: false,
        packages: vec![Package::Xtask],
        release: false,
        target: None,
        warnings_as_errors: opt.warnings_as_errors,
    };
    run_cmd(cargo.command()?)
}

/// Build docs.
fn doc(opt: &DocOpt) -> Result<()> {
    let cargo = Cargo {
        action: CargoAction::Doc { open: opt.open },
        features: Feature::more_code(),
        nightly: true,
        packages: Package::published(),
        release: false,
        target: None,
        warnings_as_errors: opt.warnings_as_errors,
    };
    run_cmd(cargo.command()?)
}

/// Build uefi-test-runner and run it in QEMU.
fn run_vm_tests(opt: &QemuOpt) -> Result<()> {
    let mut features = vec![Feature::Qemu];
    if is_ci() {
        features.push(Feature::Ci);
    }

    // Build uefi-test-runner.
    let cargo = Cargo {
        action: CargoAction::Build,
        features,
        nightly: true,
        packages: vec![Package::UefiTestRunner],
        release: opt.release,
        target: Some(opt.target),
        warnings_as_errors: false,
    };
    run_cmd(cargo.command()?)?;

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

    qemu::run_qemu(opt.target, opt, &esp_dir)
}

/// Run unit tests and doctests on the host. Most of uefi-rs is tested
/// with VM tests, but a few things like macros and data types can be
/// tested with regular tests.
fn run_host_tests() -> Result<()> {
    let cargo = Cargo {
        action: CargoAction::Test,
        features: vec![Feature::Exts],
        nightly: false,
        // Don't test uefi-services (or the packages that depend on it)
        // as it has lang items that conflict with `std`. The xtask
        // currently doesn't have any tests.
        packages: vec![Package::Uefi, Package::UefiMacros, Package::Xtask],
        release: false,
        // Use the host target so that tests can run without a VM.
        target: None,
        warnings_as_errors: false,
    };
    run_cmd(cargo.command()?)
}

fn run_ci_job(opt: CiJob) -> Result<()> {
    let warnings_as_errors = true;

    match opt {
        CiJob::BuildAArch64 => build(&BuildOpt {
            release: false,
            target: UefiArch::AArch64,
        })?,
        // TODO: get rid of that stupid is_ci thing?
        CiJob::VmTestX86_64 => run_vm_tests(&QemuOpt {
            ci: true,
            disable_kvm: true,
            headless: true,
            ovmf_dir: None,
            release: false,
            target: UefiArch::X86_64,
        })?,
        CiJob::HostTest => run_host_tests()?,
        CiJob::Lint => {
            check_formatting()?;
            clippy(&ClippyOpt {
                target: UefiArch::X86_64,
                warnings_as_errors,
            })?;
            doc(&DocOpt {
                open: false,
                warnings_as_errors: true,
            })?;
        }
        CiJob::All => {
            run_ci_job(CiJob::BuildAArch64)?;
            run_ci_job(CiJob::VmTestX86_64)?;
            run_ci_job(CiJob::HostTest)?;
            run_ci_job(CiJob::Lint)?;
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let opt = Opt::parse();

    match &opt.action {
        Action::Build(build_opt) => build(build_opt),
        Action::Clippy(clippy_opt) => clippy(clippy_opt),
        Action::Doc(doc_opt) => doc(doc_opt),
        Action::Run(qemu_opt) => run_vm_tests(qemu_opt),
        Action::Test => run_host_tests(),
        Action::Ci(job) => run_ci_job(*job),
    }
}
