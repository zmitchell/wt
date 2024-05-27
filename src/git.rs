use std::{borrow::Cow, path::Path, process::Command};

use anyhow::{anyhow, bail, Context};
use gix::Repository;
use tracing::debug;
use tracing::instrument;

use crate::{util::traceable_path, Error};
const DEFAULT_BRANCH: &str = "main";

/// Returns the global default branch name
pub fn global_default_branch_name() -> Result<String, Error> {
    let config = gix::config::File::from_globals().context("couldn't read git config")?;
    Ok(config
        .string_by_key("init.defaultBranch")
        .unwrap_or(Cow::Borrowed(DEFAULT_BRANCH.into()))
        .to_string())
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
pub fn get_worktree_branch(repo: &Repository) -> Result<String, Error> {
    repo.head_name()
        .context("couldn't get current branch")?
        .ok_or(anyhow!("couldn't get current branch"))
        .map(|b| b.to_string())
}

/// Returns the main worktree
#[instrument(skip_all, fields(starting_path = traceable_path(&starting_path)))]
pub fn get_main_worktree(starting_path: impl AsRef<Path>) -> Result<Repository, Error> {
    let starting_path = starting_path.as_ref();
    let repo = gix::discover(starting_path).context("couldn't determine current repository")?;
    let main_repo = repo.main_repo().context("couldn't find main worktree")?;
    debug!(
        path = traceable_path(main_repo.path()),
        "found main worktree"
    );
    Ok(main_repo)
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

/// Returns the path of the repo's worktree
pub fn worktree_path(repo: &Repository) -> Result<&Path, Error> {
    repo.work_dir()
        .context("main worktree was a bare repository")
}
