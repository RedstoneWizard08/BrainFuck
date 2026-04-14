//! Command-line interface for the Brainf*ck compiler.
//!
//! This module provides the CLI structure and command handling for the compiler.
//!
//! # Examples
//!
//! The CLI supports multiple subcommands:
//!
//! - `jit`: JIT compile and run a program
//! - `interpret`: Interpret a program directly
//! - `aot`: Ahead-of-time compilation to an executable
//!
//! Usage:
//! ```sh
//! bf jit program.bf
//! bf interpret program.bf
//! bf aot -o output program.bf
//! ```

use crate::{
    backend::{Backend, CompilerOptions},
    parse,
};
use anyhow::Result;
use clap::{Parser, Subcommand};
use clap_verbosity_flag::{Verbosity, WarnLevel};
use std::{fs, path::PathBuf};

#[cfg(feature = "cranelift")]
use std::str::FromStr;

#[cfg(feature = "cranelift")]
use crate::link;

/// The main CLI structure for the Brainf*ck compiler.
///
/// Provides subcommands for JIT compilation, interpretation, and ahead-of-time compilation.
///
/// # Examples
///
/// Parse and run the CLI:
/// ```no_run
/// use bf::cli::Cli;
/// use clap::Parser;
///
/// let cli = Cli::parse();
/// cli.run().unwrap();
/// ```
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
    /// Run the CLI with parsed arguments.
    pub fn run(self) -> Result<()> {
        pretty_env_logger::formatted_builder()
            .filter_level(self.verbosity.log_level_filter())
            .init();

        self.command.run()
    }
}

/// Available CLI commands.
///
/// # Examples
///
/// JIT compile a program:
/// ```sh
/// bf jit hello.bf
/// ```
///
/// Interpret a program:
/// ```sh
/// bf interpret hello.bf
/// ```
///
/// Compile to an executable with optimizations:
/// ```sh
/// bf aot -O 2 -o hello hello.bf
/// ```
///
/// Compile with a specific backend:
/// ```sh
/// bf aot -B cranelift program.bf
/// ```
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run a BrainFuck program using JIT compilation.
    /// Only works with the Cranelift compiler backend.
    #[cfg(feature = "cranelift")]
    #[command(name = "jit")]
    #[clap(aliases = &["j", "r", "run"])]
    Jit {
        /// The path to the file to run.
        file: PathBuf,

        /// Use the LLVM backend instead of Cranelift for JIT compilation.
        #[cfg(feature = "llvm")]
        #[arg(long)]
        llvm: bool,

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

        /// Use unsafe optimizations for memory access.
        #[arg(short, long = "unsafe")]
        unsafe_: bool,
    },

    /// Compile a BrainFuck program.
    #[command(name = "aot")]
    #[clap(aliases = &["c", "compile"])]
    Compile {
        /// The BrainFuck file to compile.
        file: PathBuf,

        /// The path to write the compiled binary file to.
        #[arg(short, long, default_value = "./a.out")]
        output: PathBuf,

        /// The target triple to compile for.
        /// Ignored unless using the Cranelift backend.
        #[arg(short, long)]
        target: Option<String>,

        /// Skip linking and instead output the object file.
        /// Ignored unless using the Cranelift backend.
        #[arg(short = 'c', long)]
        object: bool,

        #[command(flatten)]
        opts: CompilerOptions,
    },
}

impl Commands {
    /// Execute the selected command.
    pub fn run(self) -> Result<()> {
        match self {
            #[cfg(feature = "cranelift")]
            Self::Jit {
                file,
                opts,
                #[cfg(feature = "llvm")]
                llvm,
            } => {
                let actions = parse(&fs::read_to_string(file)?);

                let actions = if opts.opt_v1 {
                    crate::opt::v1::Optimizer::new(&opts, actions)
                        .run_all()
                        .finish_with_write()?
                } else {
                    crate::opt::v2::optimize_v2(&actions, &opts)
                };

                #[cfg(feature = "llvm")]
                {
                    if llvm {
                        crate::backend::llvm::compile(
                            target_lexicon::Triple::host(),
                            opts,
                            actions,
                            true,
                        );
                    } else {
                        crate::backend::cranelift::jit_compile_run(&actions, opts, None);
                    }
                }

                #[cfg(not(feature = "llvm"))]
                {
                    crate::backend::cranelift::jit_compile_run(&actions, opts, None);
                }
            }

            #[cfg(feature = "interp")]
            Self::Interpret {
                file,
                opts,
                unsafe_,
            } => {
                let actions = parse(&fs::read_to_string(file)?);

                let actions = if opts.opt_v1 {
                    crate::opt::v1::Optimizer::new(&opts, actions)
                        .run_all()
                        .finish_with_write()?
                } else {
                    crate::opt::v2::optimize_v2(&actions, &opts)
                };

                let stdout = std::io::stdout();
                let mut stdin = std::io::stdin();
                let mut stdout = stdout.lock();

                if unsafe_ {
                    unsafe {
                        crate::interp_unsafe::interpret(&actions, &mut stdout, &mut stdin);
                    }
                } else {
                    crate::interp::interpret(&actions, &mut stdout, &mut stdin);
                }
            }

            #[allow(unused_mut)]
            Self::Compile {
                file,
                output,
                target: _target,
                object: _object,
                mut opts,
            } => {
                let actions = parse(&fs::read_to_string(file)?);

                #[cfg(feature = "wasm")]
                if opts.backend == crate::backend::Backend::Wasm {
                    // Not yet supported on this backend
                    opts.disable(crate::backend::Optimization::Scanners);
                }

                let actions = if opts.opt_v1 {
                    crate::opt::v1::Optimizer::new(&opts, actions)
                        .run_all()
                        .finish_with_write()?
                } else {
                    crate::opt::v2::optimize_v2(&actions, &opts)
                };

                match opts.backend {
                    #[cfg(feature = "asm")]
                    Backend::Asm => {
                        fs::write(
                            output,
                            crate::backend::asm::CodeGenerator::run(&opts, &actions),
                        )?;
                    }

                    #[cfg(feature = "asm")]
                    Backend::LegacyAsm => {
                        fs::write(
                            output,
                            crate::backend::legacy_asm::CodeGenerator::run(&opts, &actions),
                        )?;
                    }

                    #[cfg(feature = "jvm")]
                    Backend::Jvm => {
                        fs::write(
                            output,
                            crate::backend::jvm::CodeGenerator::run(&opts, &actions),
                        )?;
                    }

                    #[cfg(feature = "cranelift")]
                    Backend::Cranelift => {
                        let target = _target
                            .map(|it| target_lexicon::Triple::from_str(&it).ok())
                            .flatten()
                            .unwrap_or(target_lexicon::Triple::host());

                        let obj = crate::backend::cranelift::aot_compile(&actions, &target, opts);

                        if _object {
                            fs::write(output, obj)?;
                        } else {
                            link::link_aot(obj, output, &target);
                        }
                    }

                    #[cfg(feature = "llvm")]
                    Backend::Llvm => {
                        let target = _target
                            .map(|it| target_lexicon::Triple::from_str(&it).ok())
                            .flatten()
                            .unwrap_or(target_lexicon::Triple::host());

                        let obj =
                            crate::backend::llvm::compile(target.clone(), opts, actions, false);

                        if _object {
                            fs::write(output, obj)?;
                        } else {
                            link::link_aot(obj, output, &target);
                        }
                    }

                    #[cfg(feature = "wasm")]
                    Backend::Wasm => {
                        fs::write(
                            output,
                            crate::backend::wasm::CodeGenerator::run(&opts, &actions),
                        )?;
                    }
                }
            }
        };

        Ok(())
    }
}
