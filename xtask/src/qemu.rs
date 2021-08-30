use crate::{ActionQemu, UefiTarget};
use anyhow::{anyhow, Error};
use command_run::Command;
use std::env;
use std::path::{Path, PathBuf};

struct OvmfFileNames {
    code: &'static str,
    vars: &'static str,
}

impl OvmfFileNames {
    fn new(target: UefiTarget) -> OvmfFileNames {
        match target {
            UefiTarget::Aarch64 => OvmfFileNames {
                code: "QEMU_EFI-pflash.raw",
                vars: "vars-template-pflash.raw",
            },
            UefiTarget::X86_64 => OvmfFileNames {
                code: "OVMF_CODE.fd",
                vars: "OVMF_VARS.fd",
            },
        }
    }
}

struct Ovmf {
    file_names: OvmfFileNames,
    dir: PathBuf,
}

impl Ovmf {
    fn find(repo_root: &Path, target: UefiTarget) -> Option<Ovmf> {
        let mut ovmf = Ovmf {
            file_names: OvmfFileNames::new(target),
            dir: PathBuf::new(),
        };

        let possible_paths = [
            // Test runner directory.
            repo_root.join("uefi-test-runner"),
            // Most distros, including CentOS, Fedora, Debian, and Ubuntu.
            Path::new("/usr/share/OVMF").into(),
            // Arch Linux
            Path::new("/usr/share/ovmf/x64").into(),
        ];

        for path in possible_paths {
            ovmf.dir = path;
            if ovmf.exists() {
                return Some(ovmf);
            }
        }

        None
    }

    fn code(&self) -> PathBuf {
        self.dir.join(self.file_names.code)
    }

    fn vars(&self) -> PathBuf {
        self.dir.join(self.file_names.vars)
    }

    fn exists(&self) -> bool {
        self.code().exists() && self.vars().exists()
    }
}

fn qemu_file_name_from_target(target: UefiTarget) -> &'static str {
    match target {
        UefiTarget::Aarch64 => "qemu-system-aarch64",
        UefiTarget::X86_64 => "qemu-system-x86_64",
    }
}

// TODO
fn vars_read_only_from_target(target: UefiTarget) -> &'static str {
    // The OVMF implementation for AArch64 won't boot unless the
    // vars file is writeable.
    if target == UefiTarget::Aarch64 {
        "off"
    } else {
        "on"
    }
}

/// Get the absolute path of the repo. Assumes that this executable is
/// located at <repo>/target/<buildmode>/<exename>.
fn get_repo_path() -> Result<PathBuf, Error> {
    let exe = env::current_exe()?;
    let repo_path = exe
        .parent()
        .map(|path| path.parent())
        .flatten()
        .map(|path| path.parent())
        .flatten()
        .ok_or_else(|| anyhow!("not enough parents: {}", exe.display()))?;
    Ok(repo_path.into())
}

fn get_artifact_path(repo_path: &Path, action: &ActionQemu) -> PathBuf {
    let build_mode = if action.release { "release" } else { "debug" };
    repo_path
        .join(action.target.target_triple())
        .join(build_mode)
}

pub fn run_qemu_test(action: ActionQemu) -> Result<(), Error> {
    let repo_path = get_repo_path()?;
    let artifact_path = get_artifact_path(&repo_path, &action);
    let esp_path = artifact_path.join("esp");

    let mut cmd = Command::new(qemu_file_name_from_target(action.target));

    let qemu_monitor_pipe = "qemu-monitor";
    let ovmf = Ovmf::find(&repo_path, action.target)
        .ok_or_else(|| anyhow!("failed to find OVMF files"))?;

    match action.target {
        UefiTarget::Aarch64 => {
            cmd.add_args(&[
                // Use a generic ARM environment. Sadly qemu can't
                // emulate a RPi 4 like machine though.
                "-machine",
                "virt",
                // A72 is a very generic 64-bit ARM CPU in the wild.
                "-cpu",
                "cortex-a72",
            ]);
        }
        UefiTarget::X86_64 => {
            cmd.add_args(&[
                // Use a modern machine.
                "-machine",
                "q35",
                // Multi-processor services protocol test needs exactly 4 CPUs.
                "-smp",
                "4", // Allocate some memory.
                "-m",
                "256M",
                // Enable debug features. For now these only work on x86_64.
                // Map the QEMU exit signal to port f4
                "-device",
                "isa-debug-exit,iobase=0xf4,iosize=0x04",
                // OVMF debug builds can output information to a serial `debugcon`.
                // Only enable when debugging UEFI boot:
                // "-debugcon", "file:debug.log", "-global", "isa-debugcon.iobase=0x402",
            ]);
            if !action.no_kvm {
                // Enable KVM acceleration.
                cmd.add_arg("-enable-kvm");
            }
            if action.no_reboot {
                cmd.add_arg("-no-reboot");
            }
        }
    }

    cmd.add_args(&[
        // Disable default devices. QEMU by defaults enables a ton of
        // devices which slow down boot.
        "-nodefaults",
        // Set up OVMF.
        "-drive",
        &format!(
            "if=pflash,format=raw,file={},readonly=on",
            ovmf.code().display()
        ),
        "-drive",
        &format!(
            "if=pflash,format=raw,file={},readonly={}",
            ovmf.vars().display(),
            vars_read_only_from_target(action.target),
        ),
        // Mount a local directory as a FAT partition.
        "-drive",
        &format!("format=raw,file=fat:rw:{}", esp_path.display()),
        // Connect the serial port to the host. OVMF is kind enough to connect
        // the UEFI stdout and stdin to that port too.
        "-serial",
        "stdio",
        // Map the QEMU monitor to a pair of named pipes
        "-qmp",
        &format!("pipe:{}", qemu_monitor_pipe),
    ]);

    // When running in headless mode we don't have video, but we can still have
    // QEMU emulate a display and take screenshots from it.
    cmd.add_args(&["-vga", "std"]);
    if action.headless {
        // Do not attach a window to QEMU's display
        cmd.add_args(&["-display", "none"]);
    }

    // TODO
    cmd.run()?;

    todo!();
}
