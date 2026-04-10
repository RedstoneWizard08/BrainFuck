use crate::opt::v1::Optimizer;
use log::warn;

impl<'a> Optimizer<'a> {
    pub(super) fn loop_unroll(&mut self) {
        warn!("Loop unrolling is not supported with the V1 optimizer!");
    }
}
