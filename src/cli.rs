use crate::{
    compiler::{Backend, CompilerOptions},
    interp::interpret,
    link,
    opt::Optimizer,
    parse,
};
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::{fs, path::PathBuf, str::FromStr};
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
        #[arg(short = 'c', long)]
        object: bool,

        #[command(flatten)]
        opts: CompilerOptions,
    },
}

impl Commands {
    pub fn run(self) -> Result<()> {
        match self {
            Self::Jit { file, opts } => {
                let actions = parse(&fs::read_to_string(file)?);

                let actions = Optimizer::new(&opts, actions)
                    .run_all()
                    .finish_with_write()?;

                match opts.backend {
                    Backend::Cranelift => {
                        crate::compiler::cranelift::jit_compile(&actions, opts, None)
                    }

                    #[cfg(feature = "llvm")]
                    Backend::LLVM => {
                        log::warn!(
                            "The LLVM backend is EXPERIMENTAL! Usage of it is generally discouraged, and some features may not be available!"
                        );

                        crate::compiler::llvm::jit_compile(&actions, opts)?;
                    }
                }
            }

            Self::Interpret { file, opts } => {
                let actions = parse(&fs::read_to_string(file)?);

                let actions = Optimizer::new(&opts, actions)
                    .run_all()
                    .finish_with_write()?;

                interpret(&actions, &mut std::io::stdout(), &mut std::io::stdin());
            }

            Self::Aot {
                file,
                output,
                target,
                object,
                opts,
            } => {
                let output = output.unwrap_or(PathBuf::from("a.out"));
                let actions = parse(&fs::read_to_string(file)?);

                let actions = Optimizer::new(&opts, actions)
                    .run_all()
                    .finish_with_write()?;

                let target = target
                    .map(|it| Triple::from_str(&it).ok())
                    .flatten()
                    .unwrap_or(Triple::host());

                let obj = match opts.backend {
                    Backend::Cranelift => {
                        crate::compiler::cranelift::aot_compile(&actions, &target, opts)
                    }

                    #[cfg(feature = "llvm")]
                    Backend::LLVM => {
                        log::warn!(
                            "The LLVM backend is EXPERIMENTAL! Usage of it is generally discouraged, and some features may not be available!"
                        );

                        crate::compiler::llvm::aot_compile(&actions, &target, opts)?
                    }
                };

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
