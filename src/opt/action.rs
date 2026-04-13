//! Optimized Brainf*ck action types used during compilation.
//!
//! This module defines higher-level action types that represent optimized
//! compilations of Brainf*ck operations, used by the optimizer and backends.

use serde::Serialize;

/// Value-related operations that can be performed on the current cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ValueAction {
    /// Output the current cell value
    Output,
    /// Input a value into the current cell
    Input,
    /// Add a value to the current cell
    AddValue(i64),
    /// Set the current cell to a specific value
    SetValue(i64),
    /// Bulk print operation for repeated output
    BulkPrint(i64),
}

/// Optimized Brainf*ck actions after compilation.
///
/// These are higher-level actions that represent optimized compilations
/// of basic Brainf*ck operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum OptAction {
    /// No-op action (does nothing)
    Noop,
    /// A value operation on the current cell
    Value(ValueAction),
    /// A value operation on a cell at a specific offset
    OffsetValue(ValueAction, i64),
    /// Move the pointer by a specific amount
    MovePtr(i64),
    /// Set the current cell and move the pointer
    SetAndMove(i64, i64),
    /// Increment the current cell and move the pointer
    AddAndMove(i64, i64),
    /// Copy operations from one cell to multiple target cells
    CopyLoop(Vec<(i64, i64)>),
    /// A loop containing nested optimized actions
    Loop(Vec<OptAction>),

    /// Scan operation that skips cells until a zero is found
    /// The parameter specifies how many cells to skip while scanning
    Scan(i64),
}
