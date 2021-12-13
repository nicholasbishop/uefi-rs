use crate::util::{run_cmd, Result, UefiArch, Verbose};
use clap::Parser;
use std::process::Command;

#[derive(Debug, Parser)]
pub struct QemuOpt {
    #[clap(long)]
    disable_kvm: bool,

    #[clap(long)]
    ci: bool,
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

    match arch {
        UefiArch::AArch64 => {
            todo!()
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
        }
    }

    run_cmd(cmd, verbose)
}
