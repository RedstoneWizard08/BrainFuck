use crate::{
    Action,
    backend::CompilerOptions,
    opt::{OptAction, base::OptCx},
};

pub fn optimize_v2(actions: &Vec<Action>, opts: &CompilerOptions) -> Vec<OptAction> {
    let mut cx = OptCx::new(opts.clone()).accept(actions);

    for _ in 0..opts.opt_level {
        cx.optimize(super::chain_v2::optimize_chains);
        cx.optimize(super::copy_loop_v2::copy_loop);
        cx.optimize(super::offset_v2::add_offsets);
        cx.optimize(super::sort_offset_v2::sort_offset_ops);
        cx.optimize(super::loops_v2::optimize_loops);
        cx.optimize(super::scan_v2::scanners);
        cx.optimize(super::simplify_v2::simplify);

        super::simplify_v2::simplify_start(cx.activate());
    }

    cx.finish()
}
