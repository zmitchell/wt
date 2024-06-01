use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context};
use clap::{Args, ValueHint};
use gix::Repository;

use crate::{
    git::{clone_repo, RepoLocation},
    Error,
};

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
    if let Some(path) = &args.path {
        if !path.exists() {
            return Err(anyhow!("path does not exist: {}", path.display()));
        }
    }
    let repo_loc = if args.repo.starts_with('/') {
        RepoLocation::Path(PathBuf::from(args.repo.clone()))
    } else {
        RepoLocation::Url(args.repo.as_str().try_into()?)
    };
    let temp_repo = temp_repo_checkout(
        tempfile::tempdir().context("failed to create tempdir")?,
        &repo_loc,
    )?;
    let repo_name = repo_loc
        .repo_name()
        .context("couldn't determine repo name")?;
    let default_branch = get_fresh_clone_branch_name(&temp_repo)
        .context("couldn't determine repo default branch")?;
    let project_path = if let Some(p) = &args.path {
        p.clone().join(&repo_name)
    } else {
        std::env::current_dir()
            .context("couldn't get current directory")?
            .join(&repo_name)
    };
    std::fs::create_dir_all(&project_path).context("failed to create project directory")?;
    clone_repo(&repo_loc, &project_path, Some(&default_branch))
        .context("failed to clone repository")?;
    Ok(project_path.join(default_branch))
}

/// Creates a tempororary clone of a repository in the provided directory
fn temp_repo_checkout(
    clone_repo_under: impl AsRef<Path>,
    repo_loc: &RepoLocation,
) -> Result<Repository, Error> {
    let clone_repo_under = clone_repo_under.as_ref();
    let repo_path = clone_repo(repo_loc, clone_repo_under, None::<&str>)?;
    gix::open(repo_path).context("failed to open temp checkout")
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
    fn gets_temp_repo_checkout() {
        let temp_dir = tempfile::tempdir().unwrap();

        // Create a repo that we're going to clone
        let repo_dir = temp_dir.path().join("repo_name");
        std::fs::create_dir(&repo_dir).unwrap();
        let _repo = gix::init(&repo_dir).unwrap();
        create_initial_commit(&repo_dir).unwrap();
        assert!(repo_dir.join(".git").exists());

        // Get the repo
        let clone_dir = temp_dir.path().join("clone_dir");
        let _temp_repo = temp_repo_checkout(&clone_dir, &RepoLocation::Path(repo_dir)).unwrap();
        assert!(clone_dir.join("repo_name").join(".git").exists());
    }

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
