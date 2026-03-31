use crate::opt::base::{ActiveOptCx, BfInsn, ValueInsn};
use std::collections::BTreeMap;

pub fn copy_loop(mut cx: ActiveOptCx) {
    let iter = &mut cx.insns;
    let mut no = false;
    let mut pos = 0;
    let mut changes = BTreeMap::new();

    while let Some((insn, _)) = iter.next() {
        match insn {
            BfInsn::AddAndMove(v, o) => {
                *changes.entry(pos).or_insert(0) += v;
                pos += o;
            }

            BfInsn::Value(ValueInsn::AddValue(v)) => {
                *changes.entry(pos).or_insert(0) += v;
            }

            BfInsn::MovePtr(o) => {
                pos += o;
            }

            _ => {
                no = true;
                break;
            }
        }
    }

    let c = changes.remove(&0).unwrap_or(0);

    if no || pos != 0 || c != -1 {
        return;
    }

    let changes = changes.into_iter().collect();
    let item = cx.copy_loop_arena.alloc();

    *cx.copy_loop_arena.fetch_mut(item) = changes;

    iter.clear();
    iter.add(BfInsn::CopyLoop(item));
    iter.add(BfInsn::Value(ValueInsn::SetValue(0)));
}
