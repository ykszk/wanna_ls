extern crate log;
use anyhow::Result;
use clap::{Command, CommandFactory, Parser, ValueHint};
use clap_complete::{generate, Generator, Shell};
use std::{
    path::{Path, PathBuf},
    process::ExitCode,
    time::Duration,
};

/// Wanna ls?
#[derive(Parser, Debug)]
#[command(version, about, after_help = concat!("For more info, see ",  env!("CARGO_PKG_REPOSITORY")))]
struct Args {
    #[arg(default_value = ".", value_hint = ValueHint::DirPath)]
    dir: PathBuf,
    /// Maximum number of entries
    #[arg(short = 'c', long = "count", default_value = "32")]
    max_entries: usize,
    /// Time limit in milliseconds
    #[arg(short, long, default_value = "50")]
    time_limit_ms: u64,
    /// Print the number of entries and exit
    #[arg(long)]
    entries: bool,
    /// Print shell completions
    #[arg(long, hide = true)]
    completions: Option<Shell>,
}

fn count_entries(dir: &Path) -> Result<usize> {
    let mut count = 0;
    let dir = std::fs::read_dir(dir)?;
    for entry in dir {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with('.') {
            continue;
        }
        count += 1;
    }
    Ok(count)
}

async fn async_count_entries(dir: &Path) -> Result<usize> {
    let mut count = 0;
    let mut dir = tokio::fs::read_dir(dir).await?;
    while let Some(entry) = dir.next_entry().await? {
        let name = entry.file_name();
        let name = name.to_string_lossy();
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

const EXIT_TIME_LIMIT: u8 = 2;

async fn core() -> Result<ExitCode> {
    env_logger::init();
    let args = Args::parse();

    if let Some(shell) = args.completions {
        print_completions(shell, &mut Args::command());
        return Ok(ExitCode::SUCCESS);
    }

    if args.entries {
        let count = count_entries(args.dir.as_path())?;
        println!("{count}");
        return Ok(ExitCode::SUCCESS);
    }

    let time_limit = Duration::from_millis(args.time_limit_ms);
    let count = tokio::time::timeout(time_limit, async_count_entries(args.dir.as_path())).await;
    match count {
        Ok(Ok(count)) => {
            log::debug!("Number of entries: {}", count);
            if count > args.max_entries {
                log::info!("Too many entries: ({} > {})", count, args.max_entries);
                #[allow(clippy::cast_possible_truncation)]
                let err_code = std::cmp::max(std::cmp::min(count, u8::MAX as usize) as u8, 3);
                return Ok(ExitCode::from(err_code));
            }
        }
        Ok(Err(e)) => {
            log::error!("{e}");
            return Ok(ExitCode::FAILURE);
        }
        Err(_) => {
            log::info!("Time limit exceeded");
            return Ok(ExitCode::from(EXIT_TIME_LIMIT));
        }
    };
    Ok(ExitCode::SUCCESS)
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let result = core().await;
    match result {
        Ok(code) => code,
        Err(e) => {
            log::error!("{e}");
            ExitCode::FAILURE
        }
    }
}
