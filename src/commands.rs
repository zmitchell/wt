use std::path::PathBuf;

use anyhow::Context;

use crate::{
    args::{Commands, GlobalOptions, Init},
    git::{default_branch_name, init_repo},
    Error,
};

pub fn run(cmd: &Commands, opts: &GlobalOptions) -> Result<(), Error> {
    match cmd {
        Commands::Init(args) => {
            let path = init(args)?;
            if !opts.quiet {
                println!("{}", path.display());
            }
            Ok(())
        }
    }
}

/// Creates a new worktree project
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
