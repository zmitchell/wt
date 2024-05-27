use std::{path::PathBuf, str::FromStr};

use anyhow::{anyhow, Context};
use clap::Args;
use gix::Repository;
use tracing::{debug, instrument};

use crate::{
    git::{create_branch, get_main_worktree, new_worktree, worktree_path},
    util::traceable_path,
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
    PathBuf::from_str(p).context("cannot symlink item, doesn't exist: {p}")
}

/// Creates a new worktree in the project
#[instrument]
pub fn new(args: &New) -> Result<PathBuf, Error> {
    let main_worktree =
        get_main_worktree(std::env::current_dir().context("could't get current directory")?)
            .context("couldn't locate main worktree")?;
    let new_wt_path = new_worktree_path(&main_worktree, &args.name)?;
    let (branch, needs_creating) = new_worktree_branch_name(args);
    if needs_creating {
        create_branch(&branch)?;
    }
    new_worktree(&new_wt_path, branch)?;
    for src_path in &args.symlinks {
        let suffix = src_path.strip_prefix(
            worktree_path(&main_worktree).context("couldn't get path of main worktree")?,
        )?;
        let symlink_path = new_wt_path.join(suffix);
        std::os::unix::fs::symlink(src_path, symlink_path)?;
    }
    Ok(new_wt_path)
}

/// Computes the path for the new worktree given the main worktree and the new worktree name
#[instrument(skip(main_wt, name), fields(main_wt = traceable_path(main_wt.path()), name = name.as_ref()))]
pub fn new_worktree_path(main_wt: &Repository, name: impl AsRef<str>) -> Result<PathBuf, Error> {
    let main_wt_path = worktree_path(main_wt).context("couldn't get main worktree path")?;
    let new_path = main_wt_path
        .parent()
        .ok_or(anyhow!("main worktree had no parent"))?
        .join(name.as_ref());
    debug!(
        path = traceable_path(&new_path),
        "determined new worktree location"
    );
    Ok(new_path)
}

/// Determines the branch name and whether it needs to be created
fn new_worktree_branch_name(args: &New) -> (String, bool) {
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

    use crate::commands::init::{init, Init};

    use super::*;

    #[test]
    fn branch_name_only_dir_given() {
        let args = New {
            name: "dir_name".to_string(),
            branch_name: None,
            new_branch: None,
            symlinks: vec![],
        };
        let (branch, needs_creating) = new_worktree_branch_name(&args);
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
        let (branch, needs_creating) = new_worktree_branch_name(&args);
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
        let (branch, needs_creating) = new_worktree_branch_name(&args);
        assert_eq!(branch, "new_branch");
        assert!(needs_creating);
    }

    #[test]
    fn worktree_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let main_wt_path = init(&Init {
            name: "test_proj".into(),
            path: Some(temp_dir.path().to_path_buf()),
        })
        .unwrap();
        let main_wt = gix::open(main_wt_path).unwrap();
        let new_wt_path = new_worktree_path(&main_wt, "new_wt").unwrap();
        assert_eq!(
            new_wt_path,
            temp_dir.path().join("test_proj").join("new_wt")
        );
    }
}
