use crate::opt::base::{ActiveOptCx, BfInsn};

pub fn scanners(mut cx: ActiveOptCx) {
    let iter = &mut cx.insns;
    let mut loops = Vec::new();

    while let Some((insn, _)) = iter.next() {
        if let BfInsn::Loop(item) = insn {
            let data = cx.loop_arena.fetch(item);

            if data.len() == 1
                && let BfInsn::MovePtr(m) = data[0]
            {
                iter.replace_prev(BfInsn::Scan(m));
                loops.push(item);
            }
        }
    }

    for item in loops {
        cx.drop_loop(item);
    }
}
