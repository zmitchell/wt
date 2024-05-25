use clap::{CommandFactory, Parser};
use commands::run;

use crate::args::Cli;

mod args;
mod commands;
mod git;

type Error = anyhow::Error;

fn main() -> Result<(), Error> {
    let args = Cli::parse();
    match args.command {
        Some(cmd) => {
            run(&cmd, &args.global_opts)?;
            Ok(())
        }
        None => {
            let help = Cli::command().render_help();
            println!("{help}");
            anyhow::bail!("")
        }
    }
}
