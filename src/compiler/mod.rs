pub mod cranelift;

#[cfg(feature = "llvm")]
pub mod llvm;

use clap::ValueEnum;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Default)]
pub struct CompilerOptions {
    pub unsafe_mode: bool,
    pub output_ir: Option<PathBuf>,
    pub output_asm: Option<PathBuf>,
    pub opt_level: u8,
    pub no_optimize: Vec<Optimization>,
    pub backend: Backend,
}

#[derive(
    Debug, Clone, Copy, ValueEnum, Serialize, Default, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub enum Backend {
    #[default]
    Cranelift,

    #[cfg(feature = "llvm")]
    LLVM,
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
