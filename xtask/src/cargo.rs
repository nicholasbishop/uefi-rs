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
    },
    Test {
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
        } => {
            cmd.add_args(&[
                "build",
                "--package",
                package.name(),
                "--target",
                target.target_triple(),
                "-Zbuild-std=core,compiler_builtins,alloc",
            ]);
            if release {
                cmd.add_arg("--release");
            }
        }
        CargoCommand::Test { exclude, features } => {
            cmd.add_args(&["test", "--workspace"]);
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
