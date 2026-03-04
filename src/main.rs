use std::{fs, path::PathBuf, str::FromStr};

use anyhow::Result;
use bf::{
    comp::{CompilerOptions, aot_compile, jit_compile},
    interp::interpret,
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

        /// Use unsafe pointer arithmetic.
        ///
        /// **WARNING: This can lead to faster code, but it can also lead to invalid code! Use at your own risk!**
        #[arg(long)]
        unsafe_mode: bool,

        /// The path to write CLIF (Cranelift) IR to.
        #[arg(long)]
        output_clif: Option<PathBuf>,

        /// The path to write ASM output to.
        #[arg(long)]
        output_asm: Option<PathBuf>,
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

        /// Skip linking and instead output the object file.
        #[arg(short, long)]
        object: bool,

        /// Use unsafe pointer arithmetic.
        ///
        /// **WARNING: This can lead to faster code, but it can also lead to invalid code! Use at your own risk!**
        #[arg(long)]
        unsafe_mode: bool,

        /// The path to write CLIF (Cranelift) IR to.
        #[arg(long)]
        output_clif: Option<PathBuf>,

        /// The path to write ASM output to.
        #[arg(long)]
        output_asm: Option<PathBuf>,
    },
}

pub fn main() -> Result<()> {
    pretty_env_logger::init();

    let args = Cli::parse();

    match args.command {
        Commands::Jit {
            file,
            unsafe_mode,
            output_asm,
            output_clif,
        } => {
            let actions = parse(&fs::read_to_string(file)?);
            let actions = Optimizer::new(actions).run_all().finish();

            let func = jit_compile(
                &actions,
                CompilerOptions {
                    unsafe_mode,
                    output_clif,
                    output_asm,
                },
            );

            func();
        }

        Commands::Interpret { file } => {
            let actions = parse(&fs::read_to_string(file)?);
            let actions = Optimizer::new(actions).run_all().finish();

            interpret(&actions, &mut std::io::stdout(), &mut std::io::stdin());
        }

        Commands::Aot {
            file,
            output,
            target,
            unsafe_mode,
            object,
            output_asm,
            output_clif,
        } => {
            let output = output.unwrap_or(PathBuf::from("a.out"));
            let actions = parse(&fs::read_to_string(file)?);
            let actions = Optimizer::new(actions).run_all().finish();

            let target = target
                .map(|it| Triple::from_str(&it).ok())
                .flatten()
                .unwrap_or(Triple::host());

            let obj = aot_compile(
                &actions,
                &target,
                CompilerOptions {
                    unsafe_mode,
                    output_clif,
                    output_asm,
                },
            );

            if object {
                fs::write(output, obj)?;
            } else {
                link::link_aot(obj, output, &target);
            }
        }
    }

    Ok(())
}
