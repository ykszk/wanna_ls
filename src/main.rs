extern crate log;
use anyhow::Result;
use clap::{Command, CommandFactory, Parser, ValueHint};
use clap_complete::{generate, Generator, Shell};
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
    /// Override the `too_many_entries` value in the config file
    #[arg(short, long)]
    count: Option<usize>,
    /// Print default config and exit
    #[arg(long)]
    default_config: bool,
    /// Print file system type and exit
    #[arg(long)]
    filesystem: bool,
    /// Print the number of entries and exit
    #[arg(long)]
    entries: bool,
    /// Print shell completions
    #[arg(long, hide = true)]
    completions: Option<Shell>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    denied_fs_types: Vec<String>,
    too_many_entries: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            denied_fs_types: vec![
                "nfs".to_string(),
                "cifs".to_string(),
                "smb".to_string(),
                "smb2".to_string(),
                "smbfs".to_string(),
                "webdav".to_string(),
            ],
            too_many_entries: 32,
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
    let stat = nix::sys::statfs::statfs(dir)?;
    let fs_type_name = stat.filesystem_type_name();
    Ok(fs_type_name.into())
}

fn get_config_file_path() -> Option<PathBuf> {
    let home = dirs::home_dir();
    home.map(|home| {
        home.join(".config")
            .join(env!("CARGO_PKG_NAME"))
            .join("config.toml")
    })
}

fn count_entries(dir: &Path) -> Result<usize> {
    let mut count = 0;
    let dir = std::fs::read_dir(dir)?;
    for entry in dir {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_str().unwrap();
        if name.starts_with('.') {
            continue;
        }
        count += 1;
    }
    Ok(count)
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut std::io::stdout());
}

const EXIT_FS_TYPE_DENIED: u8 = 2;

fn core() -> Result<ExitCode> {
    env_logger::init();
    let args = Args::parse();

    if let Some(shell) = args.completions {
        print_completions(shell, &mut Args::command());
        return Ok(ExitCode::SUCCESS);
    }

    if args.default_config {
        let config = Config::default();
        let config_str = toml::to_string_pretty(&config)?;
        println!("{config_str}");
        return Ok(ExitCode::SUCCESS);
    }

    if args.filesystem {
        let fs_type_name = get_fs_type_name(args.dir.as_path())?;
        println!("{fs_type_name}");
        return Ok(ExitCode::SUCCESS);
    }

    if args.entries {
        let count = count_entries(args.dir.as_path())?;
        println!("{count}");
        return Ok(ExitCode::SUCCESS);
    }

    let config_path = get_config_file_path();

    let config = if let Some(config_path) = config_path {
        if config_path.exists() {
            log::debug!("Loading config from: {}", config_path.display());
            let config_str = std::fs::read_to_string(config_path)?;
            toml::from_str(&config_str)?
        } else {
            log::debug!("Config file not found: {}", config_path.display());
            Config::default()
        }
    } else {
        log::debug!("Home path not found");
        Config::default()
    };

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

    // Count entries
    let count = count_entries(args.dir.as_path())?;
    let too_many_entries = args.count.unwrap_or(config.too_many_entries);

    log::debug!("Number of entries: {}", count);
    if count > too_many_entries {
        log::info!("Too many entries: ({} > {})", count, too_many_entries);
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
            log::error!("{e}");
            ExitCode::FAILURE
        }
    }
}
