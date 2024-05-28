use anyhow::{bail, Context};
use clap::Args;
use tracing::instrument;

use crate::{
    git::{
        delete_branch, get_main_worktree, get_worktree_branch, get_worktrees,
        global_default_branch_name, remove_worktree, sibling_worktree_path,
    },
    Error,
};

#[derive(Args, Debug, Clone)]
pub struct Remove {
    #[arg(value_name = "WT_NAME")]
    pub names: Vec<String>,

    #[arg(short, long)]
    #[arg(help = "Delete the worktree(s) without requiring confirmation")]
    pub force: bool,

    #[arg(short, long)]
    #[arg(help = "Delete the branch(es) checked out in the worktree(s)")]
    pub delete_branch: bool,
}

/// Remove one or more worktrees
#[instrument]
pub fn remove(args: &Remove) -> Result<(), Error> {
    let main_wt =
        get_main_worktree(std::env::current_dir().context("couldn't get current directory")?)
            .context("couldn't get main worktree")?;
    let to_delete = if args.names.is_empty() {
        let default_branch = global_default_branch_name().context("couldn't get default branch")?;
        let worktrees = get_worktrees(&main_wt)
            .context("couldn't get list of worktrees")?
            .into_iter()
            .filter(|name| name != &default_branch)
            .collect::<Vec<_>>();
        if worktrees.is_empty() {
            bail!("no other worktrees to remove");
        }
        inquire::MultiSelect::new("Select worktrees to remove", worktrees)
            .prompt()
            .context("failed to get selected worktrees")?
    } else {
        args.names.clone()
    };
    for name in &to_delete {
        let path = sibling_worktree_path(&main_wt, name)
            .with_context(|| format!("couldn't get path for worktree '{name}'"))?;
        let repo = gix::open(&path).with_context(|| format!("couldn't open worktree '{name}'"))?;
        let branch_ref = get_worktree_branch(&repo)
            .with_context(|| format!("couldn't get branch for worktree '{name}'"))?;
        let mut msg = format!("removed worktree '{name}'");
        remove_worktree(path).with_context(|| format!("couldn't remove worktree '{name}'"))?;
        if args.delete_branch {
            let branch_name = branch_ref.as_bstr().to_string();
            let branch_name = branch_name
                .split('/')
                .nth(2)
                .context("failed to get branch name from ref")?;
            // NOTE: you need to delete the branch from the main worktree because looking up the
            //       ref of the branch will fail in the newly-deleted worktree
            delete_branch(&main_wt, &branch_ref)
                .with_context(|| format!("couldn't delete branch '{branch_name}'"))?;
            msg.push_str(format!(" and branch '{branch_name}'").as_str());
        }
        eprintln!("{}", msg);
    }
    Ok(())
}
