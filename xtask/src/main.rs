use anyhow::{bail, Error};
use clap::{Parser, Subcommand};
use std::process::Command;

type Result<T> = std::result::Result<T, Error>;

/// Developer utility for running various tasks in uefi-rs.
#[derive(Debug, Parser)]
struct Opt {
    /// Print commands before executing them.
    #[clap(long)]
    verbose: bool,

    #[clap(subcommand)]
    action: Action,
}

#[derive(Debug, Subcommand)]
enum Action {
    Build,
    Clippy,
    Doc,
    Run,
    Test,
}

fn run_cmd(mut cmd: Command, verbose: bool) -> Result<()> {
    if verbose {
        print!("{}", cmd.get_program().to_string_lossy());
        for arg in cmd.get_args() {
            print!(" {}", arg.to_string_lossy());
        }
        println!();
    }

    let status = cmd.status()?;
    if status.success() {
        Ok(())
    } else {
        bail!("command failed: {}", status);
    }
}

fn build(opt: &Opt) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.args(&[
        "+nightly",
        "build",
        "--target",
        "x86_64-unknown-uefi",
        "-Zbuild-std=core,compiler_builtins,alloc",
        "-Zbuild-std-features=compiler-builtins-mem",
        "--workspace",
        "--exclude",
        "xtask",
    ]);
    run_cmd(cmd, opt.verbose)
}

fn main() -> Result<()> {
    let opt = &Opt::parse();

    match opt.action {
        Action::Build => build(opt),
        _ => todo!(),
    }
}
