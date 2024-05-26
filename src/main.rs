use clap::{CommandFactory, Parser};
use commands::run;
use tracing::debug;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;

use crate::commands::Cli;

mod commands;
mod git;

type Error = anyhow::Error;

fn main() -> Result<(), Error> {
    Registry::default()
        .with(EnvFilter::from_default_env())
        .with(
            HierarchicalLayer::new(2)
                .with_targets(true)
                .with_bracketed_fields(true),
        )
        .init();
    debug!("starting up");
    let args = Cli::parse();
    match args.command {
        Some(cmd) => {
            run(&cmd, &args.global_opts)?;
            Ok(())
        }
        None => {
            debug!("no command provided");
            let help = Cli::command().render_help();
            println!("{help}");
            Ok(())
        }
    }
}
