#[cfg(feature = "cranelift")]
pub mod cranelift;

#[cfg(feature = "asm")]
pub mod asm;

#[cfg(feature = "wasm")]
pub mod wasm;

use clap::ValueEnum;
use enum_display::EnumDisplay;
use serde::Serialize;
use std::path::PathBuf;

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
}

#[derive(
    Debug, Clone, Copy, Serialize, ValueEnum, PartialEq, Eq, PartialOrd, Ord, Hash, EnumDisplay,
)]
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

    /// Perform vectorization when operating on sets of numbers in a row.
    ///
    /// This requires [`Self::SetMove`] to be enabled.
    ///
    /// **Note:** Requires unsafe mode.
    Simd,

    /// Simplify Set-Then-Add operations.
    ///
    /// Some cases of this cannot be detected by [`Self::Chain`].
    SetAdd,

    /// Loop unrolling.
    LoopUnroll,
}

/// A trait for implementing custom I/O for use with the JIT compiler.
pub trait CustomIo {
    /// Get a pointer to the getchar() function.
    fn getchar(&self) -> *const u8;

    /// Get a pointer to the putchar() function.
    fn putchar(&self) -> *const u8;
}

impl CompilerOptions {
    pub fn disable(&mut self, opt: Optimization) {
        if !self.no_optimize.contains(&opt) {
            self.no_optimize.push(opt);
        }
    }
}
