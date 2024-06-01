use std::path::PathBuf;

use anyhow::Context;
use clap::Args;
use tracing::{debug, instrument};

use crate::{
    git::{create_initial_commit, global_default_branch_name},
    Error,
};

#[derive(Args, Debug, Clone)]
pub struct Init {
    /// The name of the project and the parent directory of the worktrees
    #[arg(value_name = "PROJ_NAME")]
    pub name: String,

    #[arg(short, long, value_name = "PATH")]
    #[arg(help = "The path under which to create the project in")]
    pub path: Option<PathBuf>,
}

/// Creates a new worktree project
#[instrument]
pub fn init(args: &Init) -> Result<PathBuf, Error> {
    let branch_name = global_default_branch_name()?;
    let parent_path = if let Some(p) = &args.path {
        p.clone()
    } else {
        std::env::current_dir().context("couldn't get current directory")?
    };
    let path = parent_path.join(&args.name).join(branch_name);
    std::fs::create_dir_all(&path)?;
    debug!(
        path = path.to_string_lossy().as_ref(),
        "initializing new repository"
    );
    let _repo = gix::init(&path).context("failed to init git repository")?;
    // TODO: use gix for this
    create_initial_commit(&path)?;
    Ok(path)
}
