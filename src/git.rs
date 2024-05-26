use std::{
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};

use anyhow::{bail, Context};
use tracing::instrument;
use tracing::{debug, field::Empty};

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
    let branch = if value.is_empty() {
        DEFAULT_BRANCH.to_string()
    } else {
        value.to_string()
    };
    tracing::debug!(default_branch = branch.as_str(), "found default branch");
    Ok(branch)
}

/// Creates a new repository in an existing directory
#[instrument(skip_all, fields(path = Empty))]
pub fn init_repo(path: impl AsRef<Path>) -> Result<(), Error> {
    let path = path.as_ref();
    tracing::Span::current().record("path", path.to_string_lossy().as_ref());
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

/// Gets the current commit count across all branches.
/// This is useful for detecting whether the repository is brand new.
pub fn get_commit_count() -> Result<usize, Error> {
    let output = Command::new("git")
        .args(["rev-list", "--count", "--all"])
        .output()
        .context("couldn't get commit count")?;
    if !output.status.success() {
        bail!("{}", String::from_utf8_lossy(&output.stderr));
    }
    let count = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .context("couldn't parse commit count")?;
    Ok(count)
}

/// Gets the path to the repository root
#[instrument]
pub fn get_repo_root() -> Result<PathBuf, Error> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("couldn't get repository root")?;
    if !output.status.success() {
        bail!("{}", String::from_utf8_lossy(&output.stderr).trim());
    }
    let path = PathBuf::from_str(String::from_utf8_lossy(&output.stdout).trim())?;
    debug!(
        path = path.to_string_lossy().as_ref(),
        "found repository root"
    );
    Ok(path)
}

/// Creates the initial commit in a repository
///
/// This is necessary for brand new projects to create the main branch
#[instrument]
pub fn create_initial_commit() -> Result<(), Error> {
    let output = Command::new("git")
        .args(["commit", "--allow-empty", "-m", "Initial commit"])
        .output()?;
    if !output.status.success() {
        bail!("{}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
}

/// Creates the initial commit in a repository if necessary
#[instrument]
pub fn ensure_initial_commit() -> Result<(), Error> {
    if get_commit_count()? == 0 {
        debug!("need to create initial commit");
        create_initial_commit()?;
    }
    Ok(())
}

/// Creates a new branch in the repository
#[instrument(skip_all, fields(name = name.as_ref()))]
pub fn create_branch(name: impl AsRef<str>) -> Result<(), Error> {
    let output = Command::new("git")
        .arg("branch")
        .arg(name.as_ref())
        .output()?;
    if !output.status.success() {
        bail!("{}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
}

/// Gets the main branch of the repository (likely `main` or `master`)
#[instrument]
pub fn get_repo_main_branch() -> Result<String, Error> {
    let current_dir = std::env::current_dir()?;
    if get_commit_count()? == 0 {
        return Ok(current_dir
            .file_name()
            .context("current directory had no name")?
            .to_string_lossy()
            .to_string());
    }
    std::env::set_current_dir(get_repo_root()?)?;
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()?;
    if !output.status.success() {
        bail!("{}", String::from_utf8_lossy(&output.stderr));
    }
    // Restore the original working directory
    std::env::set_current_dir(current_dir)?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Creates a new worktree at the specified path, optionally creating a new branch for the worktree
#[instrument(skip_all, fields(dir = dir.as_ref().to_string_lossy().as_ref(), branch = branch.as_ref()))]
pub fn new_worktree(dir: impl AsRef<Path>, branch: impl AsRef<str>) -> Result<(), Error> {
    let dir = dir.as_ref();
    let mut cmd = Command::new("git");
    cmd.args(["worktree", "add"]).arg(dir).arg(branch.as_ref());
    let output = cmd.output()?;
    if !output.status.success() {
        bail!("{}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
}
