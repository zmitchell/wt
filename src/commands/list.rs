use anyhow::Context;
use tracing::instrument;

use crate::{
    git::{get_main_worktree, get_worktrees, global_default_branch_name},
    Error,
};

/// Remove one or more worktrees
#[instrument]
pub fn list() -> Result<(), Error> {
    let main_wt =
        get_main_worktree(std::env::current_dir().context("couldn't get current directory")?)
            .context("couldn't get main worktree")?;
    let default_branch = global_default_branch_name().context("couldn't get default branch")?;
    let mut worktrees = get_worktrees(&main_wt)
        .context("couldn't get list of worktrees")?
        .into_iter()
        .filter(|name| name != &default_branch)
        .collect::<Vec<_>>();
    worktrees.sort();
    for name in worktrees {
        println!("{}", name);
    }
    Ok(())
}
