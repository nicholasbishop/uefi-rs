use crate::util::{run_cmd, Result, Triple};
use anyhow::bail;
use std::process::Command;

pub fn run_qemu(arch: Triple, verbose: bool) -> Result<()> {
    let qemu_exe = match arch {
        Triple::AArch64UnknownUefi => "qemu-system-aarch64",
        Triple::X86_64UnknownUefi => "qemu-system-x86_64",
        Triple::Default => bail!("invalid arch for qemu"),
    };
    let mut cmd = Command::new(qemu_exe);

    // Disable default devices.
    // QEMU by defaults enables a ton of devices which slow down boot.
    cmd.arg("-nodefaults");

    match arch {
        Triple::AArch64UnknownUefi => {
            todo!()
        }
        Triple::X86_64UnknownUefi => {
            // Use a modern machine.
            cmd.args(&["-machine", "q35"]);

            // Multi-processor services protocol test needs exactly 4 CPUs.
            cmd.args(&["-smp", "4"]);

            // Allocate some memory.
            cmd.args(&["-m", "256M"]);
        }
        Triple::Default => {}
    }

    run_cmd(cmd, verbose)
}
