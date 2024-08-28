use anyhow::{Result, Context};
use std::process::Command;
use pretty_assertions::assert_eq;

fn get_exit_code(args: &[&str]) -> Result<i32> {
    let bin = env!("CARGO_BIN_EXE_wanna_ls");
    let output = Command::new(bin)
        .args(args)
        .output()
        .context("Failed to execute command")?;
    output.status.code().context("Failed to get exit code")
}

#[test]
fn test_bin() -> Result<()> {
    let success_code = get_exit_code(&["--count", "10000", "--time-limit-ms", "100000"])
        .context("Failed to get exit code for success case")?;
    assert_eq!(success_code, 0, "Expected success code to be 0");

    let generic_error_code = get_exit_code(&["testing-for-nonexistent-path"])
        .context("Failed to get exit code for nonexistent path case")?;
    assert_eq!(generic_error_code, 1, "Expected generic error code to be 1");

    let time_limit_code = get_exit_code(&["--time-limit-ms", "0"])
        .context("Failed to get exit code for time limit case")?;
    assert_eq!(time_limit_code, 2, "Expected time limit error code to be 2");

    let too_many_entries_code = get_exit_code(&["--count", "0"])
        .context("Failed to get exit code for too many entries case")?;
    assert!(too_many_entries_code >= 3, "Expected too many entries error code to be >= 3");

    Ok(())
}