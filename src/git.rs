use std::collections::HashSet;
use std::path::PathBuf;
use std::{borrow::Cow, path::Path, process::Command};

use anyhow::{anyhow, bail, Context};
use gix::refs::{FullName, FullNameRef};
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
#[instrument(skip_all, fields(path = traceable_path(repo_path.as_ref())))]
pub fn create_initial_commit(repo_path: impl AsRef<Path>) -> Result<(), Error> {
    let mut cmd = Command::new("git");
    cmd.arg("-C");
    cmd.arg(repo_path.as_ref());
    cmd.args(["commit", "--allow-empty", "-m", "Initial commit"]);
    let output = cmd.output()?;
    if !output.status.success() {
        bail!("{}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
}

/// Creates a new branch in the repository.
///
/// Assumes you're in the project already.
#[instrument(skip_all, fields(name = name.as_ref()))]
pub fn create_branch(repo_path: impl AsRef<Path>, name: impl AsRef<str>) -> Result<(), Error> {
    let output = Command::new("git")
        .current_dir(&repo_path)
        .arg("branch")
        .arg(name.as_ref())
        .output()?;
    if !output.status.success() {
        bail!("{}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
}

/// Gets the currently checked out branch of the worktree
#[instrument]
pub fn get_worktree_branch_ref(repo: &Repository) -> Result<FullName, Error> {
    repo.head_name()
        .context("couldn't get current branch ref")?
        .ok_or(anyhow!("worktree had no HEAD"))
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
pub fn new_worktree(
    repo_path: impl AsRef<Path>,
    dir: impl AsRef<Path>,
    branch: impl AsRef<str>,
) -> Result<(), Error> {
    let dir = dir.as_ref();
    let repo_path = repo_path.as_ref();
    let mut cmd = Command::new("git");
    cmd.current_dir(repo_path);
    cmd.args(["worktree", "add"]).arg(dir).arg(branch.as_ref());
    let output = cmd.output().context("call to git-worktree failed")?;
    if !output.status.success() {
        bail!("{}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
}

/// Removes a worktree from the repository
pub fn remove_worktree(dir: impl AsRef<Path>) -> Result<(), Error> {
    let output = Command::new("git")
        .args(["worktree", "remove"])
        .arg("--force")
        .arg(dir.as_ref())
        .output()
        .context("call to git-worktree failed")?;
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

/// Returns the path for a sibling worktree
pub fn sibling_worktree_path(
    starting_wt: &Repository,
    name: impl AsRef<str>,
) -> Result<PathBuf, Error> {
    let starting_wt_path = worktree_path(starting_wt).context("couldn't get worktree path")?;
    let new_path = starting_wt_path
        .parent()
        .ok_or(anyhow!("worktree had no parent"))?
        .join(name.as_ref());
    debug!(
        path = traceable_path(&new_path),
        "determined sibling worktree location"
    );
    Ok(new_path)
}

/// Deletes the branch from the repository
pub fn delete_branch(repo: &Repository, branch_ref: &FullName) -> Result<(), Error> {
    let printable_ref_name = branch_ref.as_bstr();
    let git_ref = repo
        .find_reference(branch_ref.as_ref())
        .with_context(|| format!("couldn't find reference '{printable_ref_name}'"))?;
    git_ref
        .delete()
        .with_context(|| format!("couldn't delete git reference '{printable_ref_name}'"))
}

/// Returns a list of the worktrees other than the main worktree
pub fn get_worktrees(repo: &Repository) -> Result<Vec<String>, Error> {
    let worktrees = repo
        .worktrees()
        .context("couldn't get worktrees for repository")?;
    Ok(worktrees
        .into_iter()
        .map(|wt| wt.id().to_string())
        .collect::<Vec<_>>())
}

/// Clones the provided repository into the specified directory with the specified name
pub fn clone_repo(
    repo: impl AsRef<str>,
    clone_under: impl AsRef<Path>,
    name: Option<impl AsRef<str>>,
) -> Result<PathBuf, Error> {
    let clone_under = clone_under.as_ref();
    let repo = repo.as_ref();
    std::fs::create_dir_all(clone_under).context("couldn't create clone directory")?;
    let directories_before_clone = directories_immediately_under_path(clone_under)
        .context("couldn't get child directories before clone")?;
    let mut cmd = Command::new("git");
    cmd.current_dir(clone_under);
    cmd.args(["clone", repo]);
    if let Some(name) = name {
        cmd.arg(name.as_ref());
    }
    let output = cmd.output()?;
    if !output.status.success() {
        bail!("{}", String::from_utf8_lossy(&output.stderr));
    }
    let directories_after_clone = directories_immediately_under_path(clone_under)
        .context("couldn't get child directories after clone")?;
    let dir_diff = directories_after_clone
        .difference(&directories_before_clone)
        .cloned()
        .collect::<Vec<_>>();
    if dir_diff.len() > 1 {
        bail!("clone created more than one directory: {:?}", dir_diff);
    }
    if dir_diff.is_empty() {
        bail!("clone didn't create a directory");
    }
    Ok(dir_diff[0].clone())
}

/// Returns a set of directories immediately under the provided path
fn directories_immediately_under_path(p: impl AsRef<Path>) -> Result<HashSet<PathBuf>, Error> {
    let path = p.as_ref();
    let dirs = path
        .read_dir()
        .with_context(|| format!("couldn't read directory: {}", path.display()))?
        .filter_map(|d| match d {
            Ok(d) => {
                if d.file_type().is_ok_and(|ft| ft.is_dir()) {
                    Some(d.path())
                } else {
                    None
                }
            }
            Err(_) => None,
        })
        .collect::<HashSet<_>>();
    Ok(dirs)
}

/// Extracts the branch name from a full reference name
pub fn branch_from_ref(ref_name: &FullNameRef) -> Result<String, Error> {
    Ok(ref_name
        .as_bstr()
        .to_string()
        .split('/')
        .nth(2) // skip the "refs/heads/" prefix
        .context("failed to get branch name from ref")?
        .to_string())
}

/// Returns the name of the branch currently checked out in the repo
#[allow(dead_code)]
pub fn current_branch_name(repo: &Repository) -> Result<String, Error> {
    let branch_ref = get_worktree_branch_ref(repo).context("couldn't get ref of current branch")?;
    branch_from_ref(branch_ref.as_ref()).context("couldn't get branch name from ref")
}

#[cfg(test)]
mod test {

    use tempfile::tempdir;

    use super::*;
    use crate::commands::init::{init, Init};

    #[test]
    fn gets_branch_name_from_normal_repo() {
        let temp_dir = tempdir().unwrap();
        let repo = gix::init(temp_dir.path()).unwrap();

        // Get the default branch on this system
        let output = Command::new("git")
            .arg("-C")
            .arg(temp_dir.path())
            .arg("branch")
            .arg("--show-current")
            .output()
            .unwrap();
        let current_branch = String::from_utf8(output.stdout).unwrap().trim().to_string();

        let branch_name = current_branch_name(&repo).unwrap();
        assert_eq!(branch_name, current_branch);
    }

    #[test]
    fn reads_worktrees() {
        let temp_dir = tempfile::tempdir().unwrap();
        let init_opts = Init {
            name: "test_proj".to_string(),
            path: Some(temp_dir.path().to_path_buf()),
        };
        let main_wt_path = init(&init_opts).unwrap();
        let default_branch = global_default_branch_name().unwrap();
        let repo = gix::open(temp_dir.path().join("test_proj").join(&default_branch)).unwrap();

        // Create the worktree branch before creating the worktree
        create_branch(
            temp_dir.path().join("test_proj").join(default_branch),
            "new_worktree_branch",
        )
        .unwrap();

        // Create the new worktree
        new_worktree(
            main_wt_path,
            temp_dir.path().join("test_proj").join("new_worktree"),
            "new_worktree_branch",
        )
        .unwrap();
        let worktrees = get_worktrees(&repo).unwrap();
        assert_eq!(worktrees.len(), 1);
        assert_eq!(worktrees[0], "new_worktree".to_string());
    }

    #[test]
    fn clones_with_original_name() {
        let temp_dir = tempdir().unwrap();
        let repo_name = "repo_dir";

        // Create the repo we're going to clone
        let repo_dir = temp_dir.path().join(repo_name);
        std::fs::create_dir(&repo_dir).unwrap();
        gix::init(&repo_dir).unwrap();

        // Clone the repo
        let clone_dir = temp_dir.path().join("clone_dir");
        clone_repo(repo_dir.clone().to_string_lossy(), &clone_dir, None::<&str>).unwrap();

        assert!(clone_dir.join(repo_name).join(".git").exists());
    }

    #[test]
    fn clones_with_new_name() {
        let temp_dir = tempdir().unwrap();
        let repo_name = "repo_dir";

        // Create the repo we're going to clone
        let repo_dir = temp_dir.path().join(repo_name);
        std::fs::create_dir(&repo_dir).unwrap();
        gix::init(&repo_dir).unwrap();

        // Clone the repo
        let clone_dir = temp_dir.path().join("clone_dir");
        clone_repo(
            repo_dir.clone().to_string_lossy(),
            &clone_dir,
            Some("new_name"),
        )
        .unwrap();

        assert!(clone_dir.join("new_name").join(".git").exists());
    }
}
