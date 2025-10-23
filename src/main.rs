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
    dir: Vec<PathBuf>,
    /// Maximum number of entries
    #[arg(long = "max", default_value = "32")]
    max_entries: usize,
    /// Time limit in milliseconds
    #[arg(short, long, default_value = "50")]
    time_limit_ms: u64,
    /// Print the number of entries and exit
    #[arg(short = 'n', long, alias = "entries")]
    count: bool,
    /// Print shell completions
    #[arg(long, hide = true)]
    completions: Option<Shell>,
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

async fn count_total_entries(dirs: Vec<PathBuf>, time_limit: Duration) -> Result<usize> {
    let mut total_count = 0;
    for dir in dirs {
        let count = tokio::time::timeout(time_limit, async_count_entries(dir.as_path())).await;
        match count {
            Ok(Ok(count)) => {
                total_count += count;
            }
            Ok(Err(e)) => {
                log::error!("Error counting entries in {}: {}", dir.display(), e);
            }
            Err(_) => {
                log::info!(
                    "Time limit exceeded while counting entries in {}",
                    dir.display()
                );
            }
        }
    }
    Ok(total_count)
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

    // remove files from args.dir
    let dirs = args
        .dir
        .into_iter()
        .filter(|d| {
            if d.is_dir() {
                true
            } else {
                log::warn!("Not a directory: {}", d.display());
                false
            }
        })
        .collect::<Vec<_>>();
    if dirs.is_empty() {
        log::error!("No valid directories provided");
        return Ok(ExitCode::FAILURE);
    }

    if args.count {
        if dirs.len() > 1 {
            let mut total_count = 0;
            let mut handles = Vec::new();

            for dir in dirs {
                let handle =
                    tokio::spawn(
                        async move { (dir.clone(), async_count_entries(dir.as_path()).await) },
                    );
                handles.push(handle);
            }

            for handle in handles {
                match handle.await {
                    Ok((dir, Ok(count))) => {
                        total_count += count;
                        println!("{}: {count}", dir.display());
                    }
                    Ok((dir, Err(e))) => {
                        log::error!("Error counting entries in {}: {}", dir.display(), e);
                    }
                    Err(e) => {
                        log::error!("Task error: {}", e);
                    }
                }
            }
            println!("Total: {total_count}");
            return Ok(ExitCode::SUCCESS);
        } else {
            let count = async_count_entries(dirs[0].as_path()).await?;
            println!("{count}");
            return Ok(ExitCode::SUCCESS);
        }
    }

    let time_limit = Duration::from_millis(args.time_limit_ms);
    let count = tokio::time::timeout(time_limit, count_total_entries(dirs, time_limit)).await;
    match count {
        Ok(Ok(count)) => {
            log::debug!("Number of entries: {count}");
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
