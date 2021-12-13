use crate::util::{run_cmd, Result, UefiArch, Verbose};
use std::process::Command;

pub struct QemuOpts {
    pub arch: UefiArch,
    pub verbose: Verbose,
    pub disable_kvm: bool,
    pub ci: bool,
}

pub fn run_qemu(opts: QemuOpts) -> Result<()> {
    let qemu_exe = match opts.arch {
        UefiArch::AArch64 => "qemu-system-aarch64",
        UefiArch::X86_64 => "qemu-system-x86_64",
    };
    let mut cmd = Command::new(qemu_exe);

    // Disable default devices.
    // QEMU by defaults enables a ton of devices which slow down boot.
    cmd.arg("-nodefaults");

    match opts.arch {
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
        }
    }

    run_cmd(cmd, opts.verbose)
}
