use crate::UefiTarget;
use anyhow::Error;
use command_run::Command;

pub enum Package {
    Template,
    Uefi,
    UefiMacros,
    UefiServices,
    UefiTestRunner,
}

impl Package {
    pub fn apps() -> [Package; 2] {
        [Self::Template, Self::UefiTestRunner]
    }

    pub fn libraries() -> [Package; 3] {
        [Self::Uefi, Self::UefiMacros, Self::UefiServices]
    }

    fn name(&self) -> &str {
        match self {
            Self::Template => "uefi_app",
            Self::Uefi => "uefi",
            Self::UefiMacros => "uefi-macros",
            Self::UefiServices => "uefi-services",
            Self::UefiTestRunner => "uefi-test-runner",
        }
    }
}

pub enum CargoCommand<'a> {
    Build {
        extra_args: &'a [&'a str],
        packages: &'a [Package],
        release: bool,
        target: UefiTarget,
    },
    Clippy {
        features: &'a [&'a str],
        treat_warnings_as_errors: bool,
    },
    Doc {
        features: &'a [&'a str],
        no_deps: bool,
        open: bool,
        packages: &'a [Package],
        treat_warnings_as_errors: bool,
    },
    Format {
        check: bool,
    },
    Test {
        exclude: &'a [Package],
        features: &'a [&'a str],
        workspace: bool,
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
            extra_args,
            packages,
            release,
            target,
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
            features,
            treat_warnings_as_errors,
        } => {
            cmd.add_arg("clippy");
            add_features_args(&mut cmd, features);
            if treat_warnings_as_errors {
                cmd.add_args(&["--", "-Dwarnings"]);
            }
        }
        CargoCommand::Doc {
            features,
            no_deps,
            open,
            packages,
            treat_warnings_as_errors,
        } => {
            cmd.add_arg("doc");
            if no_deps {
                cmd.add_arg("--no-deps");
            }
            for package in packages {
                cmd.add_args(&["--package", package.name()]);
            }
            add_features_args(&mut cmd, features);
            if open {
                cmd.add_arg("--open");
            }
            if treat_warnings_as_errors {
                cmd.env.insert("RUSTDOCFLAGS".into(), "-Dwarnings".into());
            }
        }
        CargoCommand::Format { check } => {
            cmd.add_arg("fmt");
            if check {
                cmd.add_args(&["--", "--check"]);
            }
        }
        CargoCommand::Test {
            exclude,
            features,
            workspace,
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
