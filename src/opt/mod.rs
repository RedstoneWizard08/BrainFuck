//! Optimization passes for improving generated code.
//!
//! This module provides two versions of the optimizer (V1 and V2) with different
//! strategies for simplifying and improving Brainf*ck code before code generation.
//!
//! # Examples
//!
//! The optimizer can be configured to disable specific optimizations:
//!
//! ```no_run
//! use bf::backend::Optimization;
//!
//! // Create a set of optimizations to skip
//! let skip_opts = vec![Optimization::Chain];
//! ```

/// Action-level optimization utilities
pub mod action;
/// Version 1 optimization backend using chained passes
pub mod v1;
/// Version 2 optimization backend with advanced transformations
pub mod v2;
