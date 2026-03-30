use anyhow::Result;
use bf::cli::Cli;
use clap::Parser;

pub fn main() -> Result<()> {
    Cli::parse().run()?;

    Ok(())
}
