use std::path::PathBuf;

use anyhow::{anyhow, Context};
use clap::{Args, ValueHint};
use gix::Repository;

use crate::{git::clone_repo, Error};

#[derive(Args, Debug, Clone)]
pub struct Clone {
    /// The URL or path to the repository to clone
    #[arg(value_name = "REPO")]
    pub repo: String,

    #[arg(short, long, value_name = "PATH", value_hint = ValueHint::DirPath)]
    #[arg(help = "The path under which to create the project [default: current directory]")]
    pub path: Option<PathBuf>,

    #[arg(short, long, value_name = "NAME")]
    #[arg(help = "The name of the project [default: repository name]")]
    pub name: Option<String>,
}

/// Create a worktrees project by cloning a repository
pub fn init_via_clone(args: &Clone) -> Result<PathBuf, Error> {
    let current_dir = std::env::current_dir().context("couldn't get current directory")?;
    let path_to_clone_under = args.path.as_ref().map(|p| {
        if p.is_relative() {
            current_dir.join(p)
        } else {
            p.clone()
        }
    });
    if let Some(ref path) = path_to_clone_under {
        if !path.exists() {
            return Err(anyhow!("path does not exist: {}", path.display()));
        }
    }

    // Need to determine the name of the repository so we can name the parent directory of
    // all the worktrees
    let temp_dir = tempfile::tempdir().context("failed to create tempdir")?;
    let temp_repo_path = clone_repo(&args.repo, temp_dir.path(), None::<&str>)?;
    let repo_name = temp_repo_path
        .file_name()
        .ok_or(anyhow!("repo path had no file name"))?;
    let temp_repo = gix::open(&temp_repo_path).context("failed to open temp checkout")?;
    let default_branch = get_fresh_clone_branch_name(&temp_repo)
        .context("couldn't determine repo default branch")?;
    let project_path = if let Some(ref p) = path_to_clone_under {
        p.join(repo_name)
    } else {
        current_dir.join(repo_name)
    };

    std::fs::create_dir_all(&project_path).context("failed to create project directory")?;
    clone_repo(&args.repo, &project_path, Some(&default_branch))
        .context("failed to clone repository")?;
    Ok(project_path.join(default_branch))
}

/// Gets the name of the branch checked out in a fresh clone
fn get_fresh_clone_branch_name(repo: &Repository) -> Result<String, Error> {
    let branch = repo
        .branch_names()
        .into_iter()
        .next()
        .context("repo had no branches")?;
    Ok(branch.to_string())
}

#[cfg(test)]
mod test {
    use std::process::Command;

    use crate::git::create_initial_commit;

    use super::*;

    #[test]
    fn does_init_via_clone() {
        let temp_dir = tempfile::tempdir().unwrap();

        // Create a repo that we're going to clone
        let repo_dir = temp_dir.path().join("repo_name");
        std::fs::create_dir(&repo_dir).unwrap();
        let _repo = gix::init(&repo_dir).unwrap();
        create_initial_commit(&repo_dir).unwrap();
        assert!(repo_dir.join(".git").exists());

        // Get the default branch on this system
        let output = Command::new("git")
            .arg("-C")
            .arg(&repo_dir)
            .arg("branch")
            .arg("--show-current")
            .output()
            .unwrap();
        let default_branch = String::from_utf8(output.stdout).unwrap().trim().to_string();

        // Clone the repo
        let clone_dir = temp_dir.path().join("clone_dir");
        std::fs::create_dir_all(&clone_dir).unwrap();
        let project_path = init_via_clone(&Clone {
            repo: repo_dir.to_string_lossy().to_string(),
            path: Some(clone_dir.clone()),
            name: None,
        })
        .unwrap();
        assert_eq!(
            project_path,
            clone_dir.join("repo_name").join(default_branch)
        );
        assert!(project_path.exists());
    }
}
