use crate::{backend::CompilerOptions, opt::Optimizer, parse};
use anyhow::Result;
use clap::{Parser, Subcommand};
use clap_verbosity_flag::{Verbosity, WarnLevel};
use std::{fs, path::PathBuf};

#[cfg(feature = "cranelift")]
use std::str::FromStr;

#[cfg(feature = "cranelift")]
use crate::link;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// `log` verbosity options.
    #[command(flatten)]
    pub verbosity: Verbosity<WarnLevel>,

    /// The command to run.
    #[command(subcommand)]
    pub command: Commands,
}

impl Cli {
    pub fn run(self) -> Result<()> {
        pretty_env_logger::formatted_builder()
            .filter_level(self.verbosity.log_level_filter())
            .init();

        self.command.run()
    }
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
    #[cfg(feature = "wasm")]
    Wasm {
        /// The path to the file to compile.
        file: PathBuf,

        /// The path to output the WASM to.
        #[arg(short, long, default_value = "./a.wasm")]
        output: PathBuf,

        #[command(flatten)]
        opts: CompilerOptions,
    },

    /// Compile a BrainFuck program to raw assembly.
    #[command(name = "asm")]
    #[clap(aliases = &["as"])]
    #[cfg(feature = "asm")]
    Asm {
        /// The path to the file to compile.
        file: PathBuf,

        /// The path to output the assembly code to.
        #[arg(short, long, default_value = "./a.asm")]
        output: PathBuf,

        #[command(flatten)]
        opts: CompilerOptions,
    },

    /// Run a BrainFuck program using the interpreter.
    #[command(name = "interpret")]
    #[clap(aliases = &["i"])]
    #[cfg(feature = "interp")]
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
            Self::Jit { file, mut opts } => {
                opts.disable(Optimization::Scanners); // TODO

                let actions = parse(&fs::read_to_string(file)?);

                let actions = Optimizer::new(&opts, actions)
                    .run_all()
                    .finish_with_write()?;

                crate::backend::cranelift::jit_compile_run(&actions, opts, None)
            }

            #[cfg(feature = "wasm")]
            Self::Wasm {
                file,
                output,
                mut opts,
            } => {
                opts.disable(Optimization::Scanners); // TODO

                let actions = parse(&fs::read_to_string(file)?);

                let actions = Optimizer::new(&opts, actions)
                    .run_all()
                    .finish_with_write()?;

                fs::write(
                    output,
                    crate::backend::wasm::CodeGenerator::run(&opts, &actions),
                )?;
            }

            #[cfg(feature = "asm")]
            Self::Asm { file, output, opts } => {
                let actions = parse(&fs::read_to_string(file)?);

                let actions = Optimizer::new(&opts, actions)
                    .run_all()
                    .finish_with_write()?;

                fs::write(
                    output,
                    crate::backend::asm::CodeGenerator::run(&opts, &actions),
                )?;
            }

            #[cfg(feature = "interp")]
            Self::Interpret { file, mut opts } => {
                opts.disable(crate::backend::Optimization::Scanners); // TODO
                opts.disable(crate::backend::Optimization::Simd); // Unsupported with the interpreter

                let actions = parse(&fs::read_to_string(file)?);

                let actions = Optimizer::new(&opts, actions)
                    .run_all()
                    .finish_with_write()?;

                crate::interp::interpret(&actions, &mut std::io::stdout(), &mut std::io::stdin());
            }

            #[cfg(feature = "cranelift")]
            Self::Aot {
                file,
                output,
                target,
                object,
                mut opts,
            } => {
                opts.disable(Optimization::Scanners); // TODO

                let actions = parse(&fs::read_to_string(file)?);

                let actions = Optimizer::new(&opts, actions)
                    .run_all()
                    .finish_with_write()?;

                let target = target
                    .map(|it| target_lexicon::Triple::from_str(&it).ok())
                    .flatten()
                    .unwrap_or(target_lexicon::Triple::host());

                let obj = crate::backend::cranelift::aot_compile(&actions, &target, opts);

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
