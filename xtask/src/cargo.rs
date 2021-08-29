use crate::target::UefiTarget;
use anyhow::Error;
use command_run::Command;

pub enum Package {
    Template,
    UefiServices,
    UefiTestRunner,
}

impl Package {
    pub fn apps() -> [Self; 2] {
        [Self::Template, Self::UefiTestRunner]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Template => "uefi_app",
            Self::UefiServices => "uefi-services",
            Self::UefiTestRunner => "uefi-test-runner",
        }
    }
}

pub enum CargoCommand {
    Build {
        package: Package,
        target: UefiTarget,
        release: bool,
        extra_args: &'static [&'static str],
    },
    Clippy {
        all_features: bool,
        treat_warnings_as_errors: bool,
    },
    Test {
        workspace: bool,
        exclude: &'static [Package],
        features: &'static [&'static str],
    },
}

pub fn run_cargo(command: CargoCommand) -> Result<(), Error> {
    let mut cmd = Command::with_args("cargo", &["+nightly"]);
    match command {
        CargoCommand::Build {
            package,
            target,
            release,
            extra_args,
        } => {
            cmd.add_args(&[
                "build",
                "--package",
                package.name(),
                "--target",
                target.target_triple(),
            ]);
            if release {
                cmd.add_arg("--release");
            }
            cmd.add_args(extra_args);
        }
        CargoCommand::Clippy {
            all_features,
            treat_warnings_as_errors,
        } => {
            cmd.add_arg("clippy");
            if all_features {
                cmd.add_arg("--all-features");
            }
            if treat_warnings_as_errors {
                cmd.add_args(&["--", "-Dwarnings"]);
            }
        }
        CargoCommand::Test {
            workspace,
            exclude,
            features,
        } => {
            cmd.add_arg("test");
            if workspace {
                cmd.add_arg("--workspace");
            }
            for package in exclude {
                cmd.add_args(&["--exclude", package.name()]);
            }
            if !features.is_empty() {
                cmd.add_args(&["--features", &features.join(",")]);
            }
        }
    }
    cmd.run()?;
    Ok(())
}
