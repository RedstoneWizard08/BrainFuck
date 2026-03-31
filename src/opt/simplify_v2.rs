use std::collections::HashSet;

use crate::opt::base::{ActiveOptCx, BfInsn, ValueInsn};

pub fn simplify(mut cx: ActiveOptCx) {
    let iter = &mut cx.insns;

    while let Some((insn, _)) = iter.next() {
        match insn {
            BfInsn::Value(ValueInsn::AddValue(0)) | BfInsn::MovePtr(0) | BfInsn::Noop => {
                iter.remove_prev()
            }

            BfInsn::OffsetValue(it, 0) => iter.replace_prev(BfInsn::Value(it)),

            BfInsn::Value(ValueInsn::BulkPrint(0)) => {
                iter.replace_prev(BfInsn::Value(ValueInsn::Output))
            }

            _ => {}
        }
    }
}

pub fn simplify_start(mut cx: ActiveOptCx) {
    let iter = &mut cx.insns;
    let mut written = HashSet::new();
    let mut written_0 = false;

    while let Some((insn, _)) = iter.next() {
        match insn {
            BfInsn::OffsetValue(ValueInsn::AddValue(add), off) => {
                if !written.contains(&off) {
                    iter.replace_prev(BfInsn::OffsetValue(ValueInsn::SetValue(add), off));
                    written.insert(off);
                } else {
                    iter.replace_prev(BfInsn::OffsetValue(ValueInsn::AddValue(add), off));
                }
            }

            BfInsn::Value(ValueInsn::AddValue(add)) => {
                if !written_0 {
                    iter.replace_prev(BfInsn::Value(ValueInsn::SetValue(add)));
                    written_0 = true;
                } else {
                    iter.replace_prev(BfInsn::Value(ValueInsn::AddValue(add)));
                }
            }

            _ => {
                break;
            }
        }
    }
}
