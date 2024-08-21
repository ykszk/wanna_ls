extern crate log;
use anyhow::{bail, Result};
use clap::{Parser, ValueHint};
use std::{
    path::PathBuf,
    process::{Command, ExitCode},
};

/// Wanna ls?
#[derive(Parser, Debug)]
#[command(version, about, after_help = concat!("For more info, see ",  env!("CARGO_PKG_REPOSITORY")))]
struct Args {
    #[arg(default_value = ".", value_hint = ValueHint::DirPath)]
    dir: PathBuf,
}

const DENIED_FS_TYPES: [&str; 3] = ["nfs", "cifs", "smb2"];

fn core() -> Result<()> {
    env_logger::init();
    let args = Args::parse();
    let output = Command::new("stat")
        .arg("--file-system")
        .arg("--format=%T")
        .arg(args.dir)
        .output()
        .expect("failed to call `stat` command");
    if !output.status.success() {
        bail!(
            "Failed to get filesystem type: {}",
            String::from_utf8(output.stderr)?
        );
    }
    let fs_type = String::from_utf8(output.stdout).unwrap();
    log::debug!("Filesystem type: {}", fs_type);
    if DENIED_FS_TYPES.contains(&fs_type.trim()) {
        bail!("Denied filesystem type: {}", fs_type);
    }
    Ok(())
}

fn main() -> ExitCode {
    let result = core();
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            log::debug!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}
