use anyhow::Result;
use bf::cli::Cli;
use clap::Parser;

pub fn main() -> Result<()> {
    pretty_env_logger::init();
    Cli::parse().command.run()?;

    Ok(())
}
