use crate::opt::v2::base::{ActiveOptCx, BfInsn, ValueInsn};
use std::collections::BTreeMap;

pub fn sort_offset_ops(mut cx: ActiveOptCx) {
    let iter = &mut cx.insns;
    let mut buf: BTreeMap<i64, Vec<BfInsn>> = BTreeMap::new();
    let mut flag = false;

    while let Some((insn, _)) = iter.next() {
        match (insn, flag) {
            (BfInsn::OffsetValue(ValueInsn::AddValue(a), off), true) => {
                buf.entry(off)
                    .or_default()
                    .push(BfInsn::OffsetValue(ValueInsn::AddValue(a), off));
            }

            (BfInsn::OffsetValue(ValueInsn::SetValue(a), off), true) => {
                buf.entry(off)
                    .or_default()
                    .push(BfInsn::OffsetValue(ValueInsn::SetValue(a), off));
            }

            (BfInsn::OffsetValue(ValueInsn::AddValue(a), off), false) => {
                flag = true;

                buf.entry(off)
                    .or_default()
                    .push(BfInsn::OffsetValue(ValueInsn::AddValue(a), off));
            }

            (BfInsn::OffsetValue(ValueInsn::SetValue(a), off), false) => {
                flag = true;

                buf.entry(off)
                    .or_default()
                    .push(BfInsn::OffsetValue(ValueInsn::SetValue(a), off));
            }

            (_, true) => {
                flag = false;

                if buf.is_empty() {
                    continue;
                }

                let mut ins = Vec::new();

                for (_, insns) in buf {
                    ins.extend(insns);
                }

                let end = iter.pos() - 1;
                let start = end - ins.len();

                iter.swap(start..end, ins);
                buf = BTreeMap::new();
            }

            (_, false) => {}
        }
    }
}
