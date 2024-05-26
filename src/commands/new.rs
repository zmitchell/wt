use std::{path::PathBuf, str::FromStr};

use anyhow::{anyhow, Context};
use clap::Args;
use tracing::{debug, instrument};

use crate::{
    git::{create_branch, ensure_initial_commit, get_repo_root, new_worktree},
    Error,
};

#[derive(Args, Debug, Clone)]
pub struct New {
    #[arg(value_name = "DIR_NAME")]
    pub name: String,

    #[arg(short, long, group = "branch", value_name = "EXISTING_BRANCH")]
    #[arg(help = "Check out an existing branch (can't be checked out anywhere else)")]
    pub branch_name: Option<String>,

    #[arg(short, long, group = "branch", value_name = "NEW_BRANCH")]
    #[arg(help = "Create a new branch with a name different from the directory name")]
    pub new_branch: Option<String>,

    #[arg(short, long, value_name = "PATH, ...")]
    #[arg(help = "Additional files to symlink into the new worktree")]
    #[arg(value_parser = file_exists)]
    pub symlinks: Vec<PathBuf>,
}

fn file_exists(p: &str) -> Result<PathBuf, Error> {
    PathBuf::from_str(p).context("cannot symlink item: {p}")
}

/// Creates a new worktree in the project
#[instrument]
pub fn new(args: &New) -> Result<PathBuf, Error> {
    let root = get_repo_root()?;
    let new_wt_path = root
        .parent()
        .ok_or(anyhow!("main worktree had no parent"))?
        .join(&args.name);
    ensure_initial_commit()?;
    let (branch, needs_creating) = branch_name(args);
    if needs_creating {
        create_branch(&branch)?;
    }
    new_worktree(&new_wt_path, branch)?;
    for src_path in &args.symlinks {
        let suffix = src_path.strip_prefix(&root)?;
        let symlink_path = new_wt_path.join(suffix);
        std::os::unix::fs::symlink(src_path, symlink_path)?;
    }
    Ok(new_wt_path)
}

/// Determines the branch name and whether it needs to be created
fn branch_name(args: &New) -> (String, bool) {
    if let Some(ref branch_name) = args.branch_name {
        debug!(
            branch = branch_name.as_str(),
            "will check out existing branch"
        );
        (branch_name.clone(), false)
    } else if let Some(ref new_branch) = args.new_branch {
        debug!(
            branch = new_branch.as_str(),
            "will make new branch with user-specified name"
        );
        (new_branch.clone(), true)
    } else {
        debug!(
            branch = args.name.as_str(),
            "will make new branch with directory name"
        );
        (args.name.clone(), true)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn branch_name_only_dir_given() {
        let args = New {
            name: "dir_name".to_string(),
            branch_name: None,
            new_branch: None,
            symlinks: vec![],
        };
        let (branch, needs_creating) = branch_name(&args);
        assert_eq!(branch, "dir_name");
        assert!(needs_creating);
    }

    #[test]
    fn branch_name_existing_branch() {
        let args = New {
            name: "dir_name".to_string(),
            branch_name: Some("existing_branch".to_string()),
            new_branch: None,
            symlinks: vec![],
        };
        let (branch, needs_creating) = branch_name(&args);
        assert_eq!(branch, "existing_branch");
        assert!(!needs_creating);
    }

    #[test]
    fn branch_name_new_branch() {
        let args = New {
            name: "dir_name".to_string(),
            branch_name: None,
            new_branch: Some("new_branch".to_string()),
            symlinks: vec![],
        };
        let (branch, needs_creating) = branch_name(&args);
        assert_eq!(branch, "new_branch");
        assert!(needs_creating);
    }
}
