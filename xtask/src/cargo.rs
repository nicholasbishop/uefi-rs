use crate::target::UefiTarget;
use anyhow::Error;
use command_run::Command;

pub enum Package {
    Template,
    UefiServices,
    UefiTestRunner,
}

impl Package {
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
        packages: &'static [Package],
        target: UefiTarget,
        release: bool,
        extra_args: &'static [&'static str],
    },
    Clippy {
        treat_warnings_as_errors: bool,
        features: &'static [&'static str],
    },
    Test {
        workspace: bool,
        exclude: &'static [Package],
        features: &'static [&'static str],
    },
}

fn add_features_args(cmd: &mut Command, features: &[&str]) {
    if !features.is_empty() {
        cmd.add_args(&["--features", &features.join(",")]);
    }
}

pub fn run_cargo(command: CargoCommand) -> Result<(), Error> {
    let mut cmd = Command::with_args("cargo", &["+nightly"]);
    match command {
        CargoCommand::Build {
            packages,
            target,
            release,
            extra_args,
        } => {
            cmd.add_args(&["build", "--target", target.target_triple()]);
            for package in packages {
                cmd.add_args(&["--package", package.name()]);
            }
            if release {
                cmd.add_arg("--release");
            }
            cmd.add_args(extra_args);
        }
        CargoCommand::Clippy {
            treat_warnings_as_errors,
            features,
        } => {
            cmd.add_arg("clippy");
            add_features_args(&mut cmd, features);
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
            add_features_args(&mut cmd, features);
        }
    }
    cmd.run()?;
    Ok(())
}
