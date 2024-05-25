use std::{path::Path, process::Command};

use anyhow::{bail, Context};

use crate::Error;
const DEFAULT_BRANCH: &str = "main";

/// Returns the configured default branch name if it exists
pub fn default_branch_name() -> Result<String, anyhow::Error> {
    let output = Command::new("git")
        .args(["config", "init.defaultBranch"])
        .output()
        .context("failed to read default branch")?;
    let value = String::from_utf8_lossy(output.stdout.as_ref());
    let value = value.trim();
    if value.is_empty() {
        Ok(DEFAULT_BRANCH.to_string())
    } else {
        Ok(value.to_string())
    }
}

pub fn init_repo(path: impl AsRef<Path>) -> Result<(), Error> {
    let path = path.as_ref();
    if !path.exists() {
        bail!("directory doesn't exist")
    }
    let output = Command::new("git")
        .arg("init")
        .arg(path)
        .output()
        .context("couldn't init git repository")?;
    if !output.status.success() {
        bail!("{}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
}
