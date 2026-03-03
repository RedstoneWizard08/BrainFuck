use std::{fs, path::PathBuf, str::FromStr};

use anyhow::Result;
use bf::{
    comp::{aot_compile, jit_compile},
    link,
    optimizer::Optimizer,
    parse,
};
use clap::{Parser, Subcommand};
use target_lexicon::Triple;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run a BrainFuck program using JIT compilation.
    #[command(name = "jit")]
    #[clap(aliases = &["j", "r", "run"])]
    Jit {
        /// The path to the file to run.
        file: PathBuf,
    },

    /// Run a BrainFuck program using the interpreter.
    #[command(name = "interpret")]
    #[clap(aliases = &["i"])]
    Interpret {
        /// The path to the file to run.
        file: PathBuf,
    },

    /// Compile a BrainFuck program to an executable binary.
    #[command(name = "aot")]
    #[clap(aliases = &["c", "compile", "a", "b", "build"])]
    Aot {
        /// The BrainFuck file to compile.
        file: PathBuf,

        /// The path to write the compiled binary file to.
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// The target triple to compile for.
        #[arg(short, long)]
        target: Option<String>,
    },
}

pub fn main() -> Result<()> {
    pretty_env_logger::init();

    let args = Cli::parse();

    match args.command {
        Commands::Jit { file } => {
            let actions = parse(&fs::read_to_string(file)?);
            let actions = Optimizer::new(actions).run_all().finish();
            let func = jit_compile(&actions);

            func();
        }

        Commands::Interpret { file } => {
            let actions = parse(&fs::read_to_string(file)?);
            let actions = Optimizer::new(actions).run_all().finish();
            let func = jit_compile(&actions);

            func();
        }

        Commands::Aot {
            file,
            output,
            target,
        } => {
            let output = output.unwrap_or(PathBuf::from("a.out"));
            let actions = parse(&fs::read_to_string(file)?);
            let actions = Optimizer::new(actions).run_all().finish();

            let target = target
                .map(|it| Triple::from_str(&it).ok())
                .flatten()
                .unwrap_or(Triple::host());

            let obj = aot_compile(&actions, &target);

            link::link_aot(obj, output, &target);
        }
    }

    Ok(())
}
