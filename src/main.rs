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
    /// Override the `time_limit_ms` value in the config file
    #[arg(short, long)]
    time_limit_ms: Option<u64>,
    /// Print default config and exit
    #[arg(long)]
    default_config: bool,
    /// Print the number of entries and exit
    #[arg(long)]
    entries: bool,
    /// Print shell completions
    #[arg(long, hide = true)]
    completions: Option<Shell>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    time_limit_ms: u64,
    too_many_entries: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            time_limit_ms: 50,
            too_many_entries: 32,
        }
    }
}

fn get_config_file_path() -> Option<PathBuf> {
    let home = dirs::home_dir();
    home.map(|home| {
        home.join(".config")
            .join(env!("CARGO_PKG_NAME"))
            .join("config.toml")
    })
}

enum CountResult {
    Count(usize),
    TimeLimitExceeded(usize),
}

fn count_entries(dir: &Path, time_limit_ms: Option<u64>) -> Result<CountResult> {
    let mut count = 0;
    let dir = std::fs::read_dir(dir)?;
    let start = std::time::Instant::now();
    let time_limit_ms = time_limit_ms.map(std::time::Duration::from_millis);
    for entry in dir {
        if let Some(time_limit_ms) = time_limit_ms {
            if start.elapsed() > time_limit_ms {
                return Ok(CountResult::TimeLimitExceeded(count));
            }
        }
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with('.') {
            continue;
        }
        count += 1;
    }
    Ok(CountResult::Count(count))
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut std::io::stdout());
}

const EXIT_TIME_LIMIT: u8 = 2;

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

    if args.entries {
        let count = count_entries(args.dir.as_path(), None)?;
        match count {
            CountResult::Count(count) => println!("{count}"),
            CountResult::TimeLimitExceeded(count) => {
                // should not happen
                log::error!("Time limit exceeded: {}", count);
                return Ok(ExitCode::from(1));
            }
        }
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
    let time_limit_ms = args.time_limit_ms.unwrap_or(config.time_limit_ms);

    // Count entries
    let count = count_entries(args.dir.as_path(), Some(time_limit_ms))?;

    match count {
        CountResult::Count(count) => {

            let too_many_entries = args.count.unwrap_or(config.too_many_entries);

            log::debug!("Number of entries: {}", count);
            if count > too_many_entries {
                log::info!("Too many entries: ({} > {})", count, too_many_entries);
                #[allow(clippy::cast_possible_truncation)]
                let err_code = std::cmp::max(std::cmp::min(count, u8::MAX as usize) as u8, 3);
                return Ok(ExitCode::from(err_code));
            }
        }
        CountResult::TimeLimitExceeded(count) => {
            log::info!("Time limit exceeded. Counted {} entries at {} ms/entry", count, config.time_limit_ms as f64 / count as f64);
            return Ok(ExitCode::from(EXIT_TIME_LIMIT));
        }
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
