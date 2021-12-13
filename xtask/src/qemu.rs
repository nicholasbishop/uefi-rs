use crate::util::{run_cmd, Result, UefiArch, Verbose};
use anyhow::bail;
use clap::Parser;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Parser)]
pub struct QemuOpt {
    #[clap(long)]
    disable_kvm: bool,

    #[clap(long)]
    ci: bool,

    #[clap(long)]
    ovmf_dir: Option<PathBuf>,
}

/// Returns the tuple of paths to the OVMF code and vars firmware files,
/// given the directory.
fn ovmf_files(dir: &Path, arch: UefiArch) -> (PathBuf, PathBuf) {
    match arch {
        UefiArch::AArch64 => (
            dir.join("QEMU_EFI-pflash.raw"),
            dir.join("vars-template-pflash.raw"),
        ),
        UefiArch::X86_64 => (dir.join("OVMF_CODE.fd"), dir.join("OVMF_VARS.fd")),
    }
}

/// Check whether the given directory contains necessary OVMF files.
fn check_ovmf_dir(dir: &Path, arch: UefiArch) -> bool {
    let (ovmf_code, ovmf_vars) = ovmf_files(dir, arch);
    ovmf_code.is_file() && ovmf_vars.is_file()
}

/// Find path to OVMF files.
fn find_ovmf(opt: &QemuOpt, arch: UefiArch) -> Result<PathBuf> {
    // If the path is specified in the settings, use it.
    if let Some(ovmf_dir) = &opt.ovmf_dir {
        if check_ovmf_dir(ovmf_dir, arch) {
            return Ok(ovmf_dir.into());
        }
        bail!("OVMF files not found in {}", ovmf_dir.display());
    }

    // Check whether the test runner directory contains the files.
    let ovmf_dir = Path::new("uefi-test-runner");
    if check_ovmf_dir(ovmf_dir, arch) {
        return Ok(ovmf_dir.into());
    }

    #[cfg(target_os = "linux")]
    {
        let possible_paths = [
            // Most distros, including CentOS, Fedora, Debian, and Ubuntu.
            Path::new("/usr/share/OVMF"),
            // Arch Linux.
            Path::new("/usr/share/ovmf/x64"),
        ];
        for path in possible_paths {
            if check_ovmf_dir(path, arch) {
                return Ok(path.into());
            }
        }
    }

    bail!("OVMF files not found anywhere");
}

pub fn run_qemu(arch: UefiArch, opt: &QemuOpt, verbose: Verbose) -> Result<()> {
    let qemu_exe = match arch {
        UefiArch::AArch64 => "qemu-system-aarch64",
        UefiArch::X86_64 => "qemu-system-x86_64",
    };
    let mut cmd = Command::new(qemu_exe);

    // Disable default devices.
    // QEMU by defaults enables a ton of devices which slow down boot.
    cmd.arg("-nodefaults");

    let ovmf_vars_readonly;
    match arch {
        UefiArch::AArch64 => {
            // The OVMF implementation for AArch64 won't boot unless the
            // vars file is writeable.
            // TODO
            // ovmf_vars_readonly = false;
            todo!()
        }
        UefiArch::X86_64 => {
            ovmf_vars_readonly = true;

            // Use a modern machine.
            cmd.args(&["-machine", "q35"]);

            // Multi-processor services protocol test needs exactly 4 CPUs.
            cmd.args(&["-smp", "4"]);

            // Allocate some memory.
            cmd.args(&["-m", "256M"]);

            // Enable hardware-accelerated virtualization if possible.
            if !opt.disable_kvm && !opt.ci {
                cmd.arg("--enable-kvm");
            }

            // Exit instead of rebooting.
            if opt.ci {
                cmd.arg("-no-reboot");
            }
        }
    }

    // Set up OVMF.
    let (ovmf_code, ovmf_vars) = ovmf_files(&find_ovmf(opt, arch)?, arch);
    let ovmf_vars_readonly = if ovmf_vars_readonly { "on" } else { "off" };

    let mut ovmf_code_drive = OsString::from("if=pflash,format=raw,readonly=on,file=");
    ovmf_code_drive.push(ovmf_code);

    let mut ovmf_vars_drive = OsString::from("if=pflash,format=raw,readonly=");
    ovmf_vars_drive.push(ovmf_vars_readonly);
    ovmf_vars_drive.push(",file=");
    ovmf_vars_drive.push(ovmf_vars);

    cmd.arg("-drive");
    cmd.arg(ovmf_code_drive);
    cmd.arg("-drive");
    cmd.arg(ovmf_vars_drive);

    run_cmd(cmd, verbose)
}
