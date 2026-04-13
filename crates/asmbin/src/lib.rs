//! Assembly binary utilities for x86-64 code generation and manipulation.
//!
//! This crate provides low-level utilities for working with x86-64 assembly binaries,
//! including instruction encoding, register management, buffer manipulation, and binary data handling.
//!
//! # Examples
//!
//! Using the register module to work with CPU registers:
//!
//! ```no_run
//! use asmbin::reg::Reg;
//! let r_ax = Reg::Rax;
//! let width = r_ax.bit_width();
//! ```

#![feature(const_trait_impl)]

/// Buffer management utilities for binary data
pub mod buf;
/// Data structure builders for constructing binary instructions
pub mod builders;
/// Low-level binary data types and manipulation
pub mod data;
/// Example code and demonstrations
pub mod example;
/// Instruction representation and encoding
pub mod insn;
/// Iterator utilities for instruction streams
pub mod iters;
/// Register definition and manipulation
pub mod reg;
/// General utility functions and helpers
pub mod util;
