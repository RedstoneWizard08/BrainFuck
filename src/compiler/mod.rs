pub mod cranelift;

use clap::ValueEnum;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "cli", derive(clap::Args))]
pub struct CompilerOptions {
    /// The path to write codegen IR to. When using the interpreter, this is ignored.
    #[cfg_attr(feature = "cli", arg(long))]
    pub output_ir: Option<PathBuf>,

    /// The path to write ASM output to. When using the interpreter, this is ignored.
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
}

#[derive(Debug, Clone, Copy, Serialize, ValueEnum, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

    /// Optimize copy and multiplication loops down to copy and multiply instructions.
    ///
    /// **Note:** Requires unsafe mode.
    CopyLoop,

    /// Perform vectorization when operating on sets of numbers in a row.
    ///
    /// This requires [`Self::SetMove`] to be enabled.
    ///
    /// **Note:** Requires unsafe mode.
    Simd,
}

/// A trait for implementing I/O for testing with the JIT compiler.
pub trait TestingIo {
    /// Get a pointer to the getchar() function.
    fn getchar(&self) -> *const u8;

    /// Get a pointer to the putchar() function.
    fn putchar(&self) -> *const u8;
}
