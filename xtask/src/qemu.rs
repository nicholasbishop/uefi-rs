use crate::util::{run_cmd, UefiArch, Verbose};
use anyhow::{bail, Result};
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

struct OvmfPaths {
    code: PathBuf,
    vars: PathBuf,
}

impl OvmfPaths {
    fn from_dir(dir: &Path, arch: UefiArch) -> Self {
        match arch {
            UefiArch::AArch64 => Self {
                code: dir.join("QEMU_EFI-pflash.raw"),
                vars: dir.join("vars-template-pflash.raw"),
            },
            UefiArch::X86_64 => Self {
                code: dir.join("OVMF_CODE.fd"),
                vars: dir.join("OVMF_VARS.fd"),
            },
        }
    }

    fn exists(&self) -> bool {
        self.code.exists() && self.vars.exists()
    }

    /// Find path to OVMF files.
    fn find(opt: &QemuOpt, arch: UefiArch) -> Result<Self> {
        // If the path is specified in the settings, use it.
        if let Some(ovmf_dir) = &opt.ovmf_dir {
            let ovmf_paths = Self::from_dir(ovmf_dir, arch);
            if ovmf_paths.exists() {
                return Ok(ovmf_paths);
            }
            bail!("OVMF files not found in {}", ovmf_dir.display());
        }

        // Check whether the test runner directory contains the files.
        let ovmf_dir = Path::new("uefi-test-runner");
        let ovmf_paths = Self::from_dir(ovmf_dir, arch);
        if ovmf_paths.exists() {
            return Ok(ovmf_paths);
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
                let ovmf_paths = Self::from_dir(path, arch);
                if ovmf_paths.exists() {
                    return Ok(ovmf_paths);
                }
            }
        }

        bail!("OVMF files not found anywhere");
    }
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
    let ovmf_paths = OvmfPaths::find(opt, arch)?;
    let ovmf_vars_readonly = if ovmf_vars_readonly { "on" } else { "off" };

    let mut ovmf_code_drive = OsString::from("if=pflash,format=raw,readonly=on,file=");
    ovmf_code_drive.push(ovmf_paths.code);

    let mut ovmf_vars_drive = OsString::from("if=pflash,format=raw,readonly=");
    ovmf_vars_drive.push(ovmf_vars_readonly);
    ovmf_vars_drive.push(",file=");
    ovmf_vars_drive.push(ovmf_paths.vars);

    cmd.arg("-drive");
    cmd.arg(ovmf_code_drive);
    cmd.arg("-drive");
    cmd.arg(ovmf_vars_drive);

    run_cmd(cmd, verbose)
}
