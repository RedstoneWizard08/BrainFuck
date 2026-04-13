//! Version 2 optimizer with advanced transformation strategies.
//!
//! This module provides a more sophisticated optimization backend with
//! arena allocation and advanced grouping strategies.

/// Arena allocator module for efficient memory management
pub mod arena;
/// Base optimizer infrastructure
pub mod base;

/// Chain math operations optimization pass
mod chain;
/// Combination optimization passes
mod combos;
/// Copy loop pattern detection and optimization
mod copy_loop;
/// Dead code removal pass
mod dead_code;
/// Loop unrolling optimization
mod loop_unroll;
/// Loop-related optimizations
mod loops;
/// Offset-based operation optimizations
mod offsets;
/// Scanner pattern optimization
mod scan;
/// Set-and-move operation optimization
mod set_move;
/// Code simplification pass
mod simplify;
/// Sort offset operations by their offset
mod sort_offset;
/// Remove useless code at end
mod useless_end;
mod useless_ops;

use itertools::Itertools;

use crate::{
    Action,
    backend::{CompilerOptions, Optimization},
    opt::{action::OptAction, v2::base::OptCx},
};

fn optimize(cx: &mut OptCx, opt: Optimization) {
    match opt {
        Optimization::Chain => cx.optimize(chain::optimize_chains),
        Optimization::Loop => cx.optimize(loops::optimize_loops),
        Optimization::UselessOps => cx.optimize(useless_ops::optimize_useless_ops),
        Optimization::DeadCode => cx.optimize(dead_code::optimize_dead_code),
        Optimization::SetMove => cx.optimize(set_move::optimize_set_move),
        Optimization::Simplify => cx.optimize(simplify::simplify),
        Optimization::SimplifyStart => simplify::simplify_start(cx.activate()),
        Optimization::UselessEnd => cx.optimize(useless_end::optimize_useless_end),
        Optimization::Offsets => cx.optimize(offsets::add_offsets),
        Optimization::Scanners => cx.optimize(scan::scanners),
        Optimization::CopyLoop => cx.optimize(copy_loop::copy_loop),
        Optimization::SetAdd => cx.optimize(combos::optimize_set_add),
        Optimization::LoopUnroll => cx.optimize(loop_unroll::unroll_loops),
        Optimization::SortOffsetOps => cx.optimize(sort_offset::sort_offset_ops),
    }
}

pub fn optimize_v2(actions: &Vec<Action>, opts: &CompilerOptions) -> Vec<OptAction> {
    let mut cx = OptCx::new(opts.clone()).accept(actions);

    let optimizations = [
        Optimization::Chain,
        Optimization::CopyLoop,
        Optimization::Offsets,
        Optimization::SortOffsetOps,
        Optimization::Loop,
        Optimization::Scanners,
        Optimization::Simplify,
        Optimization::SimplifyStart,
    ];

    let optimizations = optimizations
        .into_iter()
        .filter(|it| !opts.no_optimize.contains(it))
        .collect_vec();

    for _ in 0..opts.opt_level {
        for opt in &optimizations {
            optimize(&mut cx, *opt);
        }
    }

    cx.finish()
}
