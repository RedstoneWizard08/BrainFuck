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
    pub fn run(self) -> Result<()> {
        match self {
            #[cfg(feature = "cranelift")]
            Self::Jit {
                file,
                mut opts,
                #[cfg(feature = "llvm")]
                llvm,
            } => {
                // Not yet supported on this backend
                opts.disable(crate::backend::Optimization::Scanners);

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
            Self::Interpret { file, mut opts } => {
                opts.disable(crate::backend::Optimization::Scanners); // TODO

                let actions = parse(&fs::read_to_string(file)?);

                let actions = if opts.opt_v1 {
                    crate::opt::v1::Optimizer::new(&opts, actions)
                        .run_all()
                        .finish_with_write()?
                } else {
                    crate::opt::v2::optimize_v2(&actions, &opts)
                };

                crate::interp::interpret(&actions, &mut std::io::stdout(), &mut std::io::stdin());
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

                #[cfg(feature = "cranelift")]
                if opts.backend == crate::backend::Backend::Cranelift {
                    // Not yet supported on this backend
                    opts.disable(crate::backend::Optimization::Scanners);
                }

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
