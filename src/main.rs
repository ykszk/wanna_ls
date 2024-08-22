extern crate log;
use anyhow::Result;
use clap::{Parser, ValueHint};
use libc::size_t;
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    process::ExitCode,
};

/// Wanna ls?
#[derive(Parser, Debug)]
#[command(version, about, after_help = concat!("For more info, see ",  env!("CARGO_PKG_REPOSITORY")))]
struct Args {
    #[arg(default_value = ".", value_hint = ValueHint::DirPath)]
    dir: PathBuf,
    #[arg(short, long, default_value = "32")]
    count: size_t,
    /// Print default config and exit
    #[arg(long)]
    config: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    denied_fs_types: Vec<String>,
    count: size_t,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            denied_fs_types: vec!["nfs".to_string(), "cifs".to_string(), "smb2".to_string()],
            count: 32,
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn get_fs_type_name(dir: &Path) -> Result<String> {
    let output = std::process::Command::new("stat")
        .arg("--file-system")
        .arg("--format=%T")
        .arg(dir)
        .output()
        .expect("failed to call `stat` command");
    if !output.status.success() {
        anyhow::bail!(
            "Failed to get filesystem type: {}",
            String::from_utf8(output.stderr)?
        );
    }
    let fs_type_name = String::from_utf8(output.stdout).unwrap();
    Ok(fs_type_name)
}

#[cfg(target_os = "macos")]
fn get_fs_type_name(dir: &Path) -> Result<String> {
    let stat = nix::sys::statfs::statfs(args.dir.as_path())?;
    let fs_type_name = stat.filesystem_type_name();
    Ok(fs_type_name)
}

const EXIT_FS_TYPE_DENIED: u8 = 2;

fn core() -> Result<ExitCode> {
    env_logger::init();
    let args = Args::parse();

    if args.config {
        let config = Config::default();
        let config_str = toml::to_string_pretty(&config)?;
        println!("{config_str}");
        return Ok(ExitCode::SUCCESS);
    }

    let config = Config::default();

    // Check filesystem type
    let fs_type_name = get_fs_type_name(args.dir.as_path())?;
    log::debug!("Filesystem type: {}", fs_type_name);
    let denied_fs_types: Vec<&str> = config
        .denied_fs_types
        .iter()
        .map(std::string::String::as_str)
        .collect();
    if denied_fs_types.contains(&fs_type_name.trim()) {
        log::info!("Denied filesystem type: {}", fs_type_name);
        return Ok(ExitCode::from(EXIT_FS_TYPE_DENIED));
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
        log::info!("Too many files: ({} > {})", count, args.count);
        #[allow(clippy::cast_possible_truncation)]
        let err_code = std::cmp::min(count, u8::MAX as usize) as u8;
        return Ok(ExitCode::from(err_code));
    }

    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    let result = core();
    match result {
        Ok(code) => code,
        Err(e) => {
            log::error!("{}", e);
            ExitCode::FAILURE
        }
    }
}
