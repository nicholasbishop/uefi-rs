use std::process::Command;

#[derive(Clone, Copy, Debug)]
pub enum Triple {
    Default,
    X86_64UnknownUefi,
    // TODO
}

#[derive(Clone, Copy, Debug)]
pub enum Package {
    Uefi,
    UefiApp,
    UefiMacros,
    UefiServices,
    UefiTestRunner,
    Xtask,
}

impl Package {
    fn as_str(self) -> &'static str {
        match self {
            Self::Uefi => "uefi",
            Self::UefiApp => "uefi_app",
            Self::UefiMacros => "uefi-macros",
            Self::UefiServices => "uefi-services",
            Self::UefiTestRunner => "uefi-test-runner",
            Self::Xtask => "xtask",
        }
    }

    /// All published packages.
    pub fn published() -> Vec<Package> {
        vec![Self::Uefi, Self::UefiMacros, Self::UefiServices]
    }

    /// All the packages except for xtask.
    pub fn all_except_xtask() -> Vec<Package> {
        vec![
            Self::Uefi,
            Self::UefiApp,
            Self::UefiMacros,
            Self::UefiServices,
            Self::UefiTestRunner,
        ]
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Features {
    /// Don't enable any extra features.
    None,

    /// Just the exts feature.
    Exts,

    /// All the features in the uefi package that enable more code.
    MoreCode,
}

impl Features {
    fn as_comma_separated_str(&self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::Exts => Some("exts"),
            Self::MoreCode => Some("alloc,exts,logger"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum CargoAction {
    Build,
    Clippy,
    Doc { open: bool },
    Test,
}

#[derive(Debug)]
pub struct Cargo {
    pub action: CargoAction,
    pub features: Features,
    pub nightly: bool,
    pub packages: Vec<Package>,
    pub target: Triple,
    pub warnings_as_errors: bool,
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
            CargoAction::Build => {
                action = "build";
                if self.warnings_as_errors {
                    extra_args.extend(&["-D", "warnings"]);
                }
            }
            CargoAction::Clippy => {
                action = "clippy";
                if self.warnings_as_errors {
                    tool_args.extend(&["-D", "warnings"]);
                }
            }
            CargoAction::Doc { open } => {
                action = "doc";
                if self.warnings_as_errors {
                    cmd.env("RUSTDOCFLAGS", "-Dwarnings");
                }
                if open {
                    extra_args.push("--open");
                }
            }
            CargoAction::Test => {
                action = "test";
                if self.warnings_as_errors {
                    panic!("cargo test can't treat warnings as errors");
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

        if self.packages.is_empty() {
            panic!("packages cannot be empty");
        }
        for package in &self.packages {
            cmd.args(&["--package", package.as_str()]);
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
