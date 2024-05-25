use clap::{Args, Parser, Subcommand};

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
    Init(Init),
}

#[derive(Args, Debug)]
pub struct Init {
    /// The name of the directory to create worktrees under
    pub name: String,
}
