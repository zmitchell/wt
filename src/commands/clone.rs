use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context};
use clap::Args;
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

    #[arg(short, long, value_name = "PATH")]
    #[arg(help = "The path under which to create the project")]
    pub path: Option<PathBuf>,

    #[arg(short, long, value_name = "NAME")]
    #[arg(help = "The name of the project")]
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
        RepoLocation::Url(args.repo.clone())
    };
    let temp_repo = temp_repo_checkout(
        tempfile::tempdir().context("failed to create tempdir")?,
        &repo_loc,
    )?;
    let (repo_name, default_branch) = get_repo_name_and_default_branch(&temp_repo)
        .context("failed to get repo name and default branch")?;
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

/// Gets the name and default branch from a repository
///
/// Note: it's assumed that the repository is a fresh clone
fn get_repo_name_and_default_branch(repo: &gix::Repository) -> Result<(String, String), Error> {
    let repo_name = repo
        .work_dir()
        .ok_or(anyhow!("repo was bare"))?
        .file_name()
        .context("invalid repo path")?
        .to_string_lossy()
        .to_string();
    let default_branch = repo
        .branch_names()
        .into_iter()
        .next() // There should only be one branch for a fresh clone
        .context("no branches")?
        .to_string();
    Ok((repo_name, default_branch))
}

/// Creates a tempororary clone of a repository in the provided directory
fn temp_repo_checkout(
    clone_repo_under: impl AsRef<Path>,
    repo_loc: &RepoLocation,
) -> Result<Repository, Error> {
    let clone_repo_under = clone_repo_under.as_ref();
    clone_repo(repo_loc, clone_repo_under, None::<&str>)?;
    let repo_path = std::fs::read_dir(clone_repo_under)
        .context("failed to open directory containing temp checkout of repository")?
        .next() // We know there's only one entry
        .context("temp checkout parent was empty")?
        .context("failed to read temp checkout of repository")?
        .path();
    gix::open(repo_path).context("failed to open temp checkout")
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
    fn gets_repo_name_and_default_branch() {
        let temp_dir = tempfile::tempdir().unwrap();

        // Create a repo that we're going to clone
        let repo_dir = temp_dir.path().join("repo_name");
        std::fs::create_dir(&repo_dir).unwrap();
        let repo = gix::init(&repo_dir).unwrap();
        create_initial_commit(&repo_dir).unwrap();
        assert!(repo_dir.join(".git").exists());

        // Get the default branch on this system
        let output = Command::new("git")
            .arg("branch")
            .arg("--show-current")
            .arg(&repo_dir)
            .output()
            .unwrap();
        let current_branch = String::from_utf8(output.stdout).unwrap().trim().to_string();

        let (repo_name, default_branch) = get_repo_name_and_default_branch(&repo).unwrap();
        assert_eq!(repo_name, "repo_name".to_string());
        assert_eq!(default_branch, current_branch);
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
            .arg("branch")
            .arg("--show-current")
            .arg(&repo_dir)
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
