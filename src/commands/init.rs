use std::path::PathBuf;

use anyhow::Context;
use clap::Args;
use tracing::instrument;

use crate::{
    git::{default_branch_name, init_repo},
    Error,
};

#[derive(Args, Debug, Clone)]
pub struct Init {
    /// The name of the project and the parent directory of the worktrees
    #[arg(value_name = "PROJ_NAME")]
    pub name: String,
}

/// Creates a new worktree project
#[instrument]
pub fn init(args: &Init) -> Result<PathBuf, Error> {
    let branch_name = default_branch_name()?;
    let path = std::env::current_dir()
        .context("couldn't get current directory")?
        .join(&args.name)
        .join(branch_name);
    std::fs::create_dir_all(&path)?;
    init_repo(&path)?;
    Ok(path)
}
