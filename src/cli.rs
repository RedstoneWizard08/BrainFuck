use crate::{
    compiler::{CompilerOptions, Optimization},
    interp::interpret,
    opt::Optimizer,
    parse,
};
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::{fs, path::PathBuf};

#[cfg(feature = "cranelift")]
use std::str::FromStr;

#[cfg(feature = "cranelift")]
use crate::link;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run a BrainFuck program using JIT compilation.
    #[cfg(feature = "cranelift")]
    #[command(name = "jit")]
    #[clap(aliases = &["j", "r", "run"])]
    Jit {
        /// The path to the file to run.
        file: PathBuf,

        #[command(flatten)]
        opts: CompilerOptions,
    },

    /// Compile a BrainFuck program to a WASM binary.
    #[command(name = "wasm")]
    #[clap(aliases = &["w", "wa"])]
    Wasm {
        /// The path to the file to compile.
        file: PathBuf,

        /// The path to output the WASM to.
        #[arg(short, long, default_value = "./a.wasm")]
        output: PathBuf,

        #[command(flatten)]
        opts: CompilerOptions,
    },

    /// Run a BrainFuck program using the interpreter.
    #[command(name = "interpret")]
    #[clap(aliases = &["i"])]
    Interpret {
        /// The path to the file to run.
        file: PathBuf,

        #[command(flatten)]
        opts: CompilerOptions,
    },

    /// Compile a BrainFuck program to an executable binary.
    #[cfg(feature = "cranelift")]
    #[command(name = "aot")]
    #[clap(aliases = &["c", "compile", "a", "b", "build"])]
    Aot {
        /// The BrainFuck file to compile.
        file: PathBuf,

        /// The path to write the compiled binary file to.
        #[arg(short, long, default_value = "./a.out")]
        output: PathBuf,

        /// The target triple to compile for.
        #[arg(short, long)]
        target: Option<String>,

        /// Skip linking and instead output the object file.
        #[arg(short = 'c', long)]
        object: bool,

        #[command(flatten)]
        opts: CompilerOptions,
    },
}

impl Commands {
    pub fn run(self) -> Result<()> {
        match self {
            #[cfg(feature = "cranelift")]
            Self::Jit { file, opts } => {
                let actions = parse(&fs::read_to_string(file)?);

                let actions = Optimizer::new(&opts, actions)
                    .run_all()
                    .finish_with_write()?;

                crate::compiler::cranelift::jit_compile_run(&actions, opts, None)
            }

            Self::Wasm { file, output, opts } => {
                let actions = parse(&fs::read_to_string(file)?);

                let actions = Optimizer::new(&opts, actions)
                    .run_all()
                    .finish_with_write()?;

                fs::write(
                    output,
                    crate::compiler::wasm::CodeGenerator::run(&opts, &actions),
                )?;
            }

            Self::Interpret { file, mut opts } => {
                opts.no_optimize.push(Optimization::Simd);

                let actions = parse(&fs::read_to_string(file)?);

                let actions = Optimizer::new(&opts, actions)
                    .run_all()
                    .finish_with_write()?;

                interpret(&actions, &mut std::io::stdout(), &mut std::io::stdin());
            }

            #[cfg(feature = "cranelift")]
            Self::Aot {
                file,
                output,
                target,
                object,
                opts,
            } => {
                let actions = parse(&fs::read_to_string(file)?);

                let actions = Optimizer::new(&opts, actions)
                    .run_all()
                    .finish_with_write()?;

                let target = target
                    .map(|it| target_lexicon::Triple::from_str(&it).ok())
                    .flatten()
                    .unwrap_or(target_lexicon::Triple::host());

                let obj = crate::compiler::cranelift::aot_compile(&actions, &target, opts);

                if object {
                    fs::write(output, obj)?;
                } else {
                    link::link_aot(obj, output, &target);
                }
            }
        };

        Ok(())
    }
}
