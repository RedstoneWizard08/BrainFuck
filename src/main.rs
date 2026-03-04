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
use ron::ser::PrettyConfig;
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

        /// The path to write the optimized tokens to.
        #[arg(long)]
        output_tokens: Option<PathBuf>,

        /// The number of optimization passes to run.
        #[arg(short = 'O', long, default_value_t = 1)]
        opt_level: u8,
    },

    /// Run a BrainFuck program using the interpreter.
    #[command(name = "interpret")]
    #[clap(aliases = &["i"])]
    Interpret {
        /// The path to the file to run.
        file: PathBuf,

        /// Enable unsafe optimizations.
        #[arg(long)]
        unsafe_mode: bool,

        /// The path to write the optimized tokens to.
        #[arg(long)]
        output_tokens: Option<PathBuf>,

        /// The number of optimization passes to run.
        #[arg(short = 'O', long, default_value_t = 1)]
        opt_level: u8,
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

        /// The path to write the optimized tokens to.
        #[arg(long)]
        output_tokens: Option<PathBuf>,

        /// The number of optimization passes to run.
        #[arg(short = 'O', long, default_value_t = 1)]
        opt_level: u8,
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
            output_tokens,
            opt_level,
        } => {
            let actions = parse(&fs::read_to_string(file)?);
            let actions = Optimizer::new(actions)
                .run_all(opt_level, unsafe_mode)
                .finish();

            if let Some(path) = output_tokens {
                fs::write(
                    path,
                    ron::ser::to_string_pretty(&actions, PrettyConfig::new())?,
                )?;
            }

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

        Commands::Interpret {
            file,
            output_tokens,
            opt_level,
            unsafe_mode,
        } => {
            let actions = parse(&fs::read_to_string(file)?);

            let actions = Optimizer::new(actions)
                .run_all(opt_level, unsafe_mode)
                .finish();

            if let Some(path) = output_tokens {
                fs::write(
                    path,
                    ron::ser::to_string_pretty(&actions, PrettyConfig::new())?,
                )?;
            }

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
            output_tokens,
            opt_level,
        } => {
            let output = output.unwrap_or(PathBuf::from("a.out"));
            let actions = parse(&fs::read_to_string(file)?);

            let actions = Optimizer::new(actions)
                .run_all(opt_level, unsafe_mode)
                .finish();

            if let Some(path) = output_tokens {
                fs::write(
                    path,
                    ron::ser::to_string_pretty(&actions, PrettyConfig::new())?,
                )?;
            }

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
