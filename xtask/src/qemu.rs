use crate::util::{run_cmd, UefiArch, Verbose};
use anyhow::{bail, Result};
use clap::Parser;
use nix::sys::stat::Mode;
use nix::unistd::mkfifo;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

#[derive(Debug, Parser)]
pub struct QemuOpt {
    #[clap(long)]
    ci: bool,

    #[clap(long)]
    disable_kvm: bool,

    #[clap(long)]
    headless: bool,

    #[clap(long)]
    ovmf_dir: Option<PathBuf>,
}

struct OvmfPaths {
    code: PathBuf,
    vars: PathBuf,
    vars_read_only: bool,
}

impl OvmfPaths {
    fn from_dir(dir: &Path, arch: UefiArch) -> Self {
        match arch {
            UefiArch::AArch64 => Self {
                code: dir.join("QEMU_EFI-pflash.raw"),
                vars: dir.join("vars-template-pflash.raw"),
                // The OVMF implementation for AArch64 won't boot unless
                // the vars file is writeable.
                vars_read_only: false,
            },
            UefiArch::X86_64 => Self {
                code: dir.join("OVMF_CODE.fd"),
                vars: dir.join("OVMF_VARS.fd"),
                vars_read_only: true,
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

fn add_pflash_args(cmd: &mut Command, file: &Path, read_only: bool) {
    // Build the argument as an OsString to avoid requiring a UTF-8 path.
    let mut arg = OsString::from("if=pflash,format=raw,readonly=");
    arg.push(if read_only { "on" } else { "off" });
    arg.push(",file=");
    arg.push(file);

    cmd.arg("-drive");
    cmd.arg(arg);
}

pub fn run_qemu(arch: UefiArch, opt: &QemuOpt, esp_dir: &Path, verbose: Verbose) -> Result<()> {
    let qemu_exe = match arch {
        UefiArch::AArch64 => "qemu-system-aarch64",
        UefiArch::X86_64 => "qemu-system-x86_64",
    };
    let mut cmd = Command::new(qemu_exe);

    // Disable default devices.
    // QEMU by defaults enables a ton of devices which slow down boot.
    cmd.arg("-nodefaults");

    match arch {
        UefiArch::AArch64 => {
            // Use a generic ARM environment. Sadly qemu can't emulate a
            // RPi 4 like machine though.
            cmd.args(&["-machine", "virt"]);

            // A72 is a very generic 64-bit ARM CPU in the wild.
            cmd.args(&["-cpu", "cortex-a72"]);
        }
        UefiArch::X86_64 => {
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

            // Map the QEMU exit signal to port f4.
            cmd.args(&["-device", "isa-debug-exit,iobase=0xf4,iosize=0x04"]);

            // OVMF debug builds can output information to a serial `debugcon`.
            // Only enable when debugging UEFI boot.
            // cmd.args(&[
            //     "-debugcon",
            //     "file:debug.log",
            //     "-global",
            //     "isa-debugcon.iobase=0x402",
            // ]);
        }
    }

    // Set up OVMF.
    let ovmf_paths = OvmfPaths::find(opt, arch)?;
    add_pflash_args(&mut cmd, &ovmf_paths.code, /*read_only=*/ true);
    add_pflash_args(&mut cmd, &ovmf_paths.vars, ovmf_paths.vars_read_only);

    // Mount a local directory as a FAT partition.
    cmd.arg("-drive");
    let mut drive_arg = OsString::from("format=raw,file=fat:rw:");
    drive_arg.push(esp_dir);
    cmd.arg(drive_arg);

    // Connect the serial port to the host. OVMF is kind enough to
    // connect the UEFI stdout and stdin to that port too.
    cmd.args(&["-serial", "stdio"]);

    let qemu_monitor_pipe = "qemu-monitor";

    // When running in headless mode we don't have video, but we can still have
    // QEMU emulate a display and take screenshots from it.
    cmd.args(&["-vga", "std"]);
    if opt.headless {
        cmd.args(&["-display", "none"]);
    }

    let tmp_dir = TempDir::new()?;
    let tmp_dir = tmp_dir.path();

    // Map the QEMU monitor to a pair of named pipes
    cmd.args(&[
        "-qmp",
        &format!("pipe:{}", tmp_dir.join(qemu_monitor_pipe).display()),
    ]);

    let monitor_input_path = tmp_dir.join(format!("{}.in", qemu_monitor_pipe));
    let monitor_output_path = tmp_dir.join(format!("{}.out", qemu_monitor_pipe));
    let fifo_mode = Mode::from_bits(0o666).unwrap();
    mkfifo(&monitor_input_path, fifo_mode)?;
    mkfifo(&monitor_output_path, fifo_mode)?;

    run_cmd(cmd, verbose)
}
