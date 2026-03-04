use anyhow::Result;
use bf::{
    compiler::{Backend, CompilerOptions, Optimization},
    interp::interpret,
    link,
    optimizer::Optimizer,
    parse,
};
use clap::{Parser, Subcommand};
use ron::ser::PrettyConfig;
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

        /// Use unsafe pointer arithmetic.
        ///
        /// **WARNING: This can lead to faster code, but it can also lead to invalid code! Use at your own risk!**
        #[arg(long)]
        unsafe_mode: bool,

        /// The path to write codegen IR to.
        #[arg(long)]
        output_ir: Option<PathBuf>,

        /// The path to write ASM output to.
        #[arg(long)]
        output_asm: Option<PathBuf>,

        /// The path to write the optimized tokens to.
        #[arg(long)]
        output_tokens: Option<PathBuf>,

        /// The number of optimization passes to run.
        #[arg(short = 'O', long, default_value_t = 1)]
        opt_level: u8,

        /// The compilation backend to use.
        #[arg(short = 'B', long, value_enum, default_value_t = Backend::default())]
        backend: Backend,

        /// Optimizations to be disabled during compilation.
        #[arg(long, alias = "--no-opt", value_enum)]
        no_optimize: Vec<Optimization>,
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
        #[arg(short = 'c', long)]
        object: bool,

        /// Use unsafe pointer arithmetic.
        ///
        /// **WARNING: This can lead to faster code, but it can also lead to invalid code! Use at your own risk!**
        #[arg(long)]
        unsafe_mode: bool,

        /// The path to write codegen IR to.
        #[arg(long)]
        output_ir: Option<PathBuf>,

        /// The path to write ASM output to.
        #[arg(long)]
        output_asm: Option<PathBuf>,

        /// The path to write the optimized tokens to.
        #[arg(long)]
        output_tokens: Option<PathBuf>,

        /// The number of optimization passes to run.
        #[arg(short = 'O', long, default_value_t = 1)]
        opt_level: u8,

        /// The compilation backend to use.
        #[arg(short = 'B', long, value_enum, default_value_t = Backend::default())]
        backend: Backend,

        /// Optimizations to be disabled during compilation.
        #[arg(long, alias = "--no-opt", value_enum)]
        no_optimize: Vec<Optimization>,
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
            output_ir,
            output_tokens,
            opt_level,
            backend,
            no_optimize,
        } => {
            let opts = CompilerOptions {
                unsafe_mode,
                output_ir,
                output_asm,
                opt_level,
                backend,
                no_optimize,
            };

            let actions = parse(&fs::read_to_string(file)?);
            let actions = Optimizer::new(actions).run_all(&opts).finish();

            if let Some(path) = output_tokens {
                fs::write(
                    path,
                    ron::ser::to_string_pretty(&actions, PrettyConfig::new())?,
                )?;
            }

            match backend {
                Backend::Cranelift => {
                    let func = bf::compiler::cranelift::jit_compile(&actions, opts);

                    func();
                }

                #[cfg(feature = "llvm")]
                Backend::LLVM => {
                    log::warn!(
                        "The LLVM backend is EXPERIMENTAL! Usage of it is generally discouraged, and some features may not be available!"
                    );

                    bf::compiler::llvm::jit_compile(&actions, opts)?;
                }
            }
        }

        Commands::Interpret {
            file,
            output_tokens,
            opt_level,
            unsafe_mode,
        } => {
            let opts = CompilerOptions {
                unsafe_mode,
                opt_level,

                ..Default::default()
            };

            let actions = parse(&fs::read_to_string(file)?);
            let actions = Optimizer::new(actions).run_all(&opts).finish();

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
            output_ir,
            output_tokens,
            opt_level,
            backend,
            no_optimize,
        } => {
            let opts = CompilerOptions {
                unsafe_mode,
                output_ir,
                output_asm,
                opt_level,
                backend,
                no_optimize,
            };

            let output = output.unwrap_or(PathBuf::from("a.out"));
            let actions = parse(&fs::read_to_string(file)?);
            let actions = Optimizer::new(actions).run_all(&opts).finish();

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

            let obj = match backend {
                Backend::Cranelift => bf::compiler::cranelift::aot_compile(&actions, &target, opts),

                #[cfg(feature = "llvm")]
                Backend::LLVM => {
                    log::warn!(
                        "The LLVM backend is EXPERIMENTAL! Usage of it is generally discouraged, and some features may not be available!"
                    );

                    bf::compiler::llvm::aot_compile(&actions, &target, opts)?
                }
            };

            if object {
                fs::write(output, obj)?;
            } else {
                link::link_aot(obj, output, &target);
            }
        }
    }

    Ok(())
}
