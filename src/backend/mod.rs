#[cfg(feature = "asm")]
pub mod asm;

#[cfg(feature = "asm")]
pub mod legacy_asm;

#[cfg(feature = "cranelift")]
pub mod cranelift;

#[cfg(feature = "wasm")]
pub mod wasm;

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

    /// The compiler backend to use.
    #[cfg_attr(feature = "cli", arg(short = 'B', long, value_enum, default_value_t = Backend::default()))]
    pub backend: Backend,

    /// Use the legacy V1 optimization backend.
    #[cfg_attr(feature = "cli", arg(long))]
    pub opt_v1: bool,
}

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

    /// The WASM codegen backend.
    /// Outputs a WASM binary.
    #[cfg(feature = "wasm")]
    #[cfg_attr(
        all(not(feature = "asm"), not(feature = "cranelift"), feature = "wasm"),
        default
    )]
    Wasm,
}

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
