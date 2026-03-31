use crate::opt::base::{ActiveOptCx, BfInsn, ValueInsn};

pub fn optimize_loops(mut cx: ActiveOptCx) {
    let iter = &mut cx.insns;
    let mut remove = Vec::new();

    while let Some((insn, _)) = iter.next() {
        if let BfInsn::Loop(item) = insn {
            let data = cx.loop_arena.fetch(item);

            if data.is_empty() {
                iter.remove_prev();
            } else if data.len() == 1 && matches!(data[0], BfInsn::Value(ValueInsn::AddValue(_))) {
                iter.replace_prev(BfInsn::Value(ValueInsn::SetValue(0)));
            } else if data.len() == 2
                && matches!(data[0], BfInsn::CopyLoop(_))
                && (data[1] == BfInsn::Value(ValueInsn::SetValue(0))
                    || matches!(data[1], BfInsn::OffsetValue(ValueInsn::SetValue(0), _)))
            {
                let mut data = data.to_vec();

                data.pop();
                iter.insert_buf(data.into_iter());
                remove.push(item);
            }

            while let Some((BfInsn::Loop(it), _)) = iter.next() {
                remove.push(it);
            }
        } else if let BfInsn::CopyLoop(item) = insn {
            let data = cx.copy_loop_arena.fetch(item);

            if data.is_empty() {
                drop(data);

                iter.replace_prev(BfInsn::Value(ValueInsn::SetValue(0)));
                cx.copy_loop_arena.drop_item(item);
            }
        }
    }

    for item in remove {
        cx.drop_loop(item);
    }
}
