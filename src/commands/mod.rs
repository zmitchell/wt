use clap::{Args, Parser, Subcommand};
use tracing::instrument;

use crate::Error;

pub mod clone;
pub mod init;
pub mod list;
pub mod new;
pub mod rm;

pub use init::init;
pub use new::new;

use self::{
    clone::{init_via_clone, Clone},
    init::Init,
    list::list,
    new::New,
    rm::{remove, Remove},
};

#[derive(Parser, Debug)]
#[command(version)]
#[command(about = "Utility for managing git worktrees")]
pub struct Cli {
    #[command(flatten)]
    pub global_opts: GlobalOptions,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Clone, Args)]
pub struct GlobalOptions {
    #[arg(short, long)]
    #[arg(help = "Silences all output")]
    pub quiet: bool,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(about = "Create a new worktree project")]
    #[command(long_about = include_str!("../long_help/init.md"))]
    Init(Init),
    #[command(about = "Create a new worktree")]
    #[command(long_about = include_str!("../long_help/new.md"))]
    New(New),
    #[command(about = "Remove one or more worktrees")]
    #[command(alias = "rm")]
    Remove(Remove),
    #[command(about = "List worktrees")]
    #[command(alias = "ls")]
    List,
    #[command(about = "Create a worktrees project by cloning a repository")]
    Clone(Clone),
}

#[instrument(skip(cmd))]
pub fn run(cmd: &Commands, opts: &GlobalOptions) -> Result<(), Error> {
    match cmd {
        Commands::Init(args) => {
            let path = init(args)?;
            if !opts.quiet {
                println!("{}", path.display());
            }
            Ok(())
        }
        Commands::New(args) => {
            let path = new(args)?;
            if !opts.quiet {
                println!("{}", path.display());
            }
            Ok(())
        }
        Commands::Remove(args) => {
            remove(args)?;
            Ok(())
        }
        Commands::List => {
            list()?;
            Ok(())
        }
        Commands::Clone(args) => {
            init_via_clone(args)?;
            Ok(())
        }
    }
}
