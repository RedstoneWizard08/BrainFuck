//! Code generation backends for various target platforms.
//!
//! This module provides implementations of multiple code generation backends
//! for compiling Brainf*ck programs to different target platforms including
//! native assembly, LLVM IR, Cranelift, WebAssembly, and JVM bytecode.

#[cfg(feature = "asm")]
/// Modern ASM backend for direct ELF executable generation
pub mod asm;

#[cfg(feature = "llvm")]
/// LLVM IR code generation backend
pub mod llvm;

#[cfg(feature = "asm")]
/// Legacy ASM backend for x86-64 assembly output
pub mod legacy_asm;

#[cfg(feature = "cranelift")]
/// Cranelift codegen backend for object file generation
pub mod cranelift;

#[cfg(feature = "wasm")]
/// WebAssembly code generation backend
pub mod wasm;

#[cfg(feature = "jvm")]
/// JVM bytecode generation backend
pub mod jvm;

use enum_display::EnumDisplay;
use serde::Serialize;
use std::path::PathBuf;

/// Compilation options controlling code generation and optimization behavior.
///
/// This structure holds all user-configurable compilation parameters including
/// output paths, optimization settings, target backend selection, and tape size.
///
/// # Examples
///
/// Create default compilation options:
///
/// ```no_run
/// use bf::backend::CompilerOptions;
///
/// let opts = CompilerOptions::default();
/// ```
///
/// Create options with a specific backend and optimization level:
///
/// ```no_run
/// use bf::backend::{CompilerOptions, Backend};
///
/// let mut opts = CompilerOptions::default();
/// opts.backend = Backend::Asm;
/// opts.opt_level = 2;
/// ```
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "cli", derive(clap::Args))]
pub struct CompilerOptions {
    /// The path to write codegen IR to. When using the interpreter, this is ignored.
    #[cfg(feature = "cranelift")]
    #[cfg_attr(feature = "cli", arg(long))]
    pub output_ir: Option<PathBuf>,

    /// The path to write ASM output to. When using the interpreter, this is ignored.
    #[cfg(feature = "cranelift")]
    #[cfg_attr(feature = "cli", arg(long))]
    pub output_asm: Option<PathBuf>,

    /// The path to write the optimized tokens to.
    #[cfg_attr(feature = "cli", arg(long))]
    pub output_tokens: Option<PathBuf>,

    /// The number of optimization passes to run.
    #[cfg_attr(feature = "cli", arg(short = 'O', long, default_value_t = 1))]
    pub opt_level: u8,

    /// Optimizations to be disabled during compilation.
    #[cfg_attr(feature = "cli", arg(long, alias = "--no-opt", value_enum))]
    pub no_optimize: Vec<Optimization>,

    /// The size of the tape.
    ///
    /// For performance reasons, the interpreter ignores this option.
    #[cfg_attr(feature = "cli", arg(short = 'T', long, default_value_t = 65536))]
    pub tape_size: usize,

    /// Disable I/O operations. Useful for benchmarking and profiling.
    #[cfg_attr(feature = "cli", arg(short = 'N', long))]
    pub no_io: bool,

    /// The compiler backend to use.
    #[cfg_attr(feature = "cli", arg(short = 'B', long, value_enum, default_value_t = Backend::default()))]
    pub backend: Backend,

    /// Use the legacy V1 optimization backend.
    #[cfg_attr(feature = "cli", arg(long))]
    pub opt_v1: bool,
}

/// Available code generation backends for Brainf*ck compilation.
///
/// Each backend targets a different platform or intermediate representation
///
/// # Examples
///
/// The ASM backend generates native ELF executables directly:
/// ```no_run
/// use bf::backend::Backend;
/// let backend = Backend::Asm;
/// ```
///
/// The WASM backend generates WebAssembly modules:
/// ```no_run
/// use bf::backend::Backend;
/// let backend = Backend::Wasm;
/// ```
#[derive(
    Debug, Clone, Copy, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash, EnumDisplay, Default,
)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
pub enum Backend {
    /// The modern ASM backend.
    /// Outputs an executable (ELF) file directly.
    #[cfg(feature = "asm")]
    #[cfg_attr(feature = "asm", default)]
    Asm,

    /// The legacy ASM backend.
    /// Outputs an x86_64 assembly source file.
    #[cfg(feature = "asm")]
    LegacyAsm,

    /// The Cranelift codegen backend.
    /// Outputs an object file.
    #[cfg(feature = "cranelift")]
    #[cfg_attr(all(not(feature = "asm"), feature = "cranelift"), default)]
    Cranelift,

    /// The LLVM codegen backend.
    /// Outputs an object file.
    #[cfg(feature = "llvm")]
    #[cfg_attr(
        all(not(feature = "asm"), not(feature = "cranelift"), feature = "llvm"),
        default
    )]
    Llvm,

    /// The WASM codegen backend.
    /// Outputs a WASM binary.
    #[cfg(feature = "wasm")]
    #[cfg_attr(
        all(
            not(feature = "asm"),
            not(feature = "cranelift"),
            not(feature = "llvm"),
            feature = "wasm"
        ),
        default
    )]
    Wasm,

    /// The JVM codegen backend.
    /// Outputs a class file.
    #[cfg(feature = "jvm")]
    Jvm,
}

/// Available optimization passes for improving generated code.
///
/// Different optimization passes focus on different aspects of code quality
/// and can be independently enabled or disabled.
///
/// # Examples
///
/// Disable the Chain optimization:
///
/// ```no_run
/// use bf::backend::{CompilerOptions, Optimization};
///
/// let mut opts = CompilerOptions::default();
/// opts.disable(Optimization::Chain);
/// ```
///
/// Disable multiple optimizations:
///
/// ```no_run
/// use bf::backend::{CompilerOptions, Optimization};
///
/// let mut opts = CompilerOptions::default();
/// opts.disable(Optimization::UselessOps);
/// opts.disable(Optimization::DeadCode);
/// ```
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash, EnumDisplay)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
pub enum Optimization {
    /// Chain math and shift operations.
    Chain,

    /// Remove useless loops, and turn single-instruction loops (`[+]`, `[-]`) into a simple 0.
    Loop,

    /// Remove operations that do nothing.
    UselessOps,

    /// Remove duplicate loops and unreachable code.
    DeadCode,

    /// Combine set and move operations into a single instruction.
    SetMove,

    /// Simplify redundant code, and remove add/move zero operations.
    Simplify,

    /// Simplify starting value instructions to be set instead of add.
    SimplifyStart,

    /// Remove instructions at the end that don't affect the final output.
    UselessEnd,

    /// Replace groups of instructions followed by moves with single instructions.
    Offsets,

    /// Replace scanners with optimized variants.
    Scanners,

    /// Optimize copy and multiplication loops down to copy and multiply instructions.
    ///
    /// **Note:** Requires unsafe mode.
    CopyLoop,

    /// Simplify Set-Then-Add operations.
    ///
    /// Some cases of this cannot be detected by [`Self::Chain`].
    SetAdd,

    /// Loop unrolling.
    LoopUnroll,

    /// Sort offseted operations by their offset.
    ///
    /// Opens up the possibility of further grouping and optimizations.
    ///
    /// Only supported with the V2 optimizer.
    SortOffsetOps,
}

/// Custom I/O handler trait for use with JIT compilation.
///
/// Allows providing custom implementations of I/O operations,
/// particularly useful for sandboxing and profiling.
///
/// # Examples
///
/// Implementing custom I/O:
///
/// ```no_run
/// use bf::backend::CustomIo;
///
/// struct MyIO;
///
/// impl CustomIo for MyIO {
///     fn getchar(&self) -> *const u8 {
///         // Return pointer to custom getchar implementation
///         unimplemented!()
///     }
///
///     fn putchar(&self) -> *const u8 {
///         // Return pointer to custom putchar implementation
///         unimplemented!()
///     }
/// }
/// ```
pub trait CustomIo {
    /// Get a pointer to the getchar() function.
    /// This function should read a single byte from input.
    fn getchar(&self) -> *const u8;

    /// Get a pointer to the putchar() function.
    /// This function should write a single byte to output.
    fn putchar(&self) -> *const u8;
}

impl CompilerOptions {
    /// Disable a specific optimization pass.
    ///
    /// # Arguments
    ///
    /// * `opt` - The optimization to disable
    ///
    /// # Examples
    ///
    /// Disable multiple optimizations:
    ///
    /// ```
    /// use bf::backend::{CompilerOptions, Optimization};
    ///
    /// let mut opts = CompilerOptions::default();
    /// opts.disable(Optimization::Chain);
    /// opts.disable(Optimization::DeadCode);
    /// assert!(opts.no_optimize.contains(&Optimization::Chain));
    /// ```
    pub fn disable(&mut self, opt: Optimization) {
        if !self.no_optimize.contains(&opt) {
            self.no_optimize.push(opt);
        }
    }
}
