use std::process::Command;

#[derive(Clone, Copy, Debug)]
pub enum Triple {
    Default,
    X86_64UnknownUefi,
    // TODO
}

#[derive(Clone, Copy, Debug)]
pub enum Packages {
    /// All the published packages.
    Published,

    /// All the packages except for xtask.
    EverythingExceptXtask,

    /// Just the xtask package.
    Xtask,
}

#[derive(Clone, Copy, Debug)]
pub enum Features {
    None,

    /// All the features in the uefi package that enable more code.
    MoreCode,
}

impl Features {
    fn as_comma_separated_str(&self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::MoreCode => Some("alloc,exts,logger"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum CargoAction {
    Build,
    Clippy,
    Doc { open: bool },
}

#[derive(Debug)]
pub struct Cargo {
    pub action: CargoAction,
    pub features: Features,
    pub nightly: bool,
    pub packages: Packages,
    pub target: Triple,
}

impl Cargo {
    pub fn command(&self) -> Command {
        let mut cmd = Command::new("cargo");
        if self.nightly {
            cmd.arg("+nightly");
        }

        let action;
        let mut extra_args: Vec<&str> = Vec::new();
        let mut tool_args: Vec<&str> = Vec::new();
        match self.action {
            CargoAction::Build => action = "build",
            CargoAction::Clippy => {
                action = "clippy";
                // Treat all warnings as errors.
                tool_args.extend(&["-D", "warnings"]);
            }
            CargoAction::Doc { open } => {
                action = "doc";
                // Treat all warnings as errors.
                cmd.env("RUSTDOCFLAGS", "-Dwarnings");
                if open {
                    extra_args.push("--open");
                }
            }
        };
        cmd.arg(action);

        match self.target {
            Triple::Default => {}
            Triple::X86_64UnknownUefi => {
                cmd.args(&[
                    "--target",
                    "x86_64-unknown-uefi",
                    "-Zbuild-std=core,compiler_builtins,alloc",
                    "-Zbuild-std-features=compiler-builtins-mem",
                ]);
            }
        }

        match self.packages {
            Packages::Published => {
                cmd.args(&["--workspace", "--exclude", "uefi_app", "--exclude", "xtask"]);
            }
            Packages::EverythingExceptXtask => {
                cmd.args(&["--workspace", "--exclude", "xtask"]);
            }
            Packages::Xtask => {
                cmd.args(&["--package", "xtask"]);
            }
        }

        if let Some(features) = self.features.as_comma_separated_str() {
            cmd.args(&["--features", features]);
        }

        cmd.args(extra_args);

        if !tool_args.is_empty() {
            cmd.arg("--");
            cmd.args(tool_args);
        }

        cmd
    }
}
