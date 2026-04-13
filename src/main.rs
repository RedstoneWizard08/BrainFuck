//! Brainf*ck compiler binary entry point.
//!
//! This executable provides the command-line interface for the Brainf*ck compiler.
//!
//! # Examples
//!
//! Compile a Brainf*ck program to an executable:
//!
//! ```sh
//! cargo run -- aot -o output.elf program.bf
//! ```
//!
//! Interpret a program directly:
//!
//! ```sh
//! cargo run -- interpret program.bf
//! ```
//!
//! JIT compile and run a program:
//!
//! ```sh
//! cargo run -- jit program.bf
//! ```

use anyhow::Result;
use bf::cli::Cli;
use clap::Parser;

/// Main entry point for the Brainf*ck compiler.
///
/// Parses command-line arguments and executes the compiler according to the specified options.
///
/// # Returns
///
/// `Ok(())` on successful compilation
pub fn main() -> Result<()> {
    Cli::parse().run()?;

    Ok(())
}
