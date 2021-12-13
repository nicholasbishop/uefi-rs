use crate::util::{run_cmd, Arch, Result};
use std::process::Command;

pub fn run_qemu(arch: Arch, verbose: bool) -> Result<()> {
    let qemu_exe = match arch {
        Arch::AArch64UnknownUefi => "qemu-system-aarch64",
        Arch::X86_64UnknownUefi => "qemu-system-x86_64",
    };
    let mut cmd = Command::new(qemu_exe);

    // Disable default devices.
    // QEMU by defaults enables a ton of devices which slow down boot.
    cmd.arg("-nodefaults");

    match arch {
        Arch::AArch64UnknownUefi => {
            todo!()
        }
        Arch::X86_64UnknownUefi => {
            // Use a modern machine.
            cmd.args(&["-machine", "q35"]);

            // Multi-processor services protocol test needs exactly 4 CPUs.
            cmd.args(&["-smp", "4"]);

            // Allocate some memory.
            cmd.args(&["-m", "256M"]);
        }
    }

    run_cmd(cmd, verbose)
}
