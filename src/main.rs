extern crate log;
use anyhow::{bail, Result};
use clap::{Parser, ValueHint};
use libc::size_t;
use std::{path::PathBuf, process::ExitCode};

/// Wanna ls?
#[derive(Parser, Debug)]
#[command(version, about, after_help = concat!("For more info, see ",  env!("CARGO_PKG_REPOSITORY")))]
struct Args {
    #[arg(default_value = ".", value_hint = ValueHint::DirPath)]
    dir: PathBuf,
    #[arg(short, long, default_value = "32")]
    count: size_t,
}

const DENIED_FS_TYPES: [&str; 3] = ["nfs", "cifs", "smb2"];

fn core() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    // Check filesystem type
    #[cfg(not(target_os = "macos"))]
    {
        let output = std::process::Command::new("stat")
            .arg("--file-system")
            .arg("--format=%T")
            .arg(args.dir.as_path())
            .output()
            .expect("failed to call `stat` command");
        if !output.status.success() {
            bail!(
                "Failed to get filesystem type: {}",
                String::from_utf8(output.stderr)?
            );
        }
        let fs_type_name = String::from_utf8(output.stdout).unwrap();
        log::debug!("Filesystem type: {}", fs_type_name);
        if DENIED_FS_TYPES.contains(&fs_type_name.trim()) {
            bail!("Denied filesystem type: {}", fs_type_name);
        }
    }

    #[cfg(target_os = "macos")]
    {
        let stat = nix::sys::statfs::statfs(args.dir.as_path())?;
        let fs_type_name = stat.filesystem_type_name();
        log::debug!("Filesystem type: {}", fs_type_name);
        if DENIED_FS_TYPES.contains(&fs_type_name.trim()) {
            bail!("Denied filesystem type: {}", fs_type_name);
        }
    }

    // Count files
    let mut count = 0;
    let dir = std::fs::read_dir(args.dir)?;
    for entry in dir {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_str().unwrap();
        if name.starts_with('.') {
            continue;
        }
        count += 1;
    }

    log::debug!("Number of files: {}", count);
    if count > args.count {
        bail!("Too many files: ({} > {})", count, args.count);
    }

    Ok(())
}

fn main() -> ExitCode {
    let result = core();
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            log::info!("{}", e);
            ExitCode::FAILURE
        }
    }
}
