use crate::opt::base::{ActiveOptCx, BfInsn, ValueInsn};

trait Offsettable {
    fn offsettable(&self) -> bool;
    fn offset(&self) -> i64;
}

impl Offsettable for BfInsn {
    fn offsettable(&self) -> bool {
        match self {
            Self::Value(_)
            | Self::OffsetValue(_, _)
            | Self::AddAndMove(_, _)
            | Self::SetAndMove(_, _)
            | Self::MovePtr(_) => true,
            _ => false,
        }
    }

    fn offset(&self) -> i64 {
        match self {
            Self::OffsetValue(_, o) => *o,
            _ => panic!("Action was not offsettable!"),
        }
    }
}

pub fn add_offsets(mut cx: ActiveOptCx) {
    let mut start = 0;
    let iter = &mut cx.insns;

    while start < iter.len() {
        while start < iter.len() && !iter[start].offsettable() {
            start += 1;
        }

        if start >= iter.len() {
            break;
        }

        let mut end = start;

        while end < iter.len() && iter[end].offsettable() {
            end += 1;
        }

        // the block [i..j) is a sequence of operations that can be replaced with offsets

        let mut new_block = Vec::new();
        let mut cur_block = Vec::new();
        let mut ptr_delta = 0;

        for pos in start..end {
            let op = &iter[pos];

            match op {
                BfInsn::Value(ValueInsn::Input) => {
                    new_block.append(&mut cur_block);
                    new_block.push(BfInsn::OffsetValue(ValueInsn::Input, ptr_delta));
                }

                BfInsn::OffsetValue(ValueInsn::Input, o) => {
                    new_block.append(&mut cur_block);
                    new_block.push(BfInsn::OffsetValue(ValueInsn::Input, ptr_delta + o));
                }

                BfInsn::Value(ValueInsn::Output) => {
                    new_block.append(&mut cur_block);
                    new_block.push(BfInsn::OffsetValue(ValueInsn::Output, ptr_delta));
                }

                BfInsn::OffsetValue(ValueInsn::Output, o) => {
                    new_block.append(&mut cur_block);
                    new_block.push(BfInsn::OffsetValue(ValueInsn::Output, ptr_delta + o));
                }

                BfInsn::Value(ValueInsn::BulkPrint(n)) => {
                    new_block.append(&mut cur_block);

                    new_block.push(BfInsn::OffsetValue(ValueInsn::BulkPrint(*n), ptr_delta));
                }

                BfInsn::OffsetValue(ValueInsn::BulkPrint(n), o) => {
                    new_block.extend(cur_block);
                    cur_block = Vec::new();

                    new_block.push(BfInsn::OffsetValue(ValueInsn::BulkPrint(*n), ptr_delta + o));
                }

                BfInsn::Value(ValueInsn::AddValue(c)) => {
                    cur_block.push(BfInsn::OffsetValue(ValueInsn::AddValue(*c), ptr_delta));
                }

                BfInsn::OffsetValue(ValueInsn::AddValue(c), o) => {
                    cur_block.push(BfInsn::OffsetValue(ValueInsn::AddValue(*c), ptr_delta + o));
                }

                BfInsn::AddAndMove(c, o) => {
                    cur_block.push(BfInsn::OffsetValue(ValueInsn::AddValue(*c), ptr_delta));

                    ptr_delta += o;
                }

                BfInsn::Value(ValueInsn::SetValue(c)) => {
                    cur_block.push(BfInsn::OffsetValue(ValueInsn::SetValue(*c), ptr_delta));
                }

                BfInsn::OffsetValue(ValueInsn::SetValue(c), o) => {
                    cur_block.push(BfInsn::OffsetValue(ValueInsn::SetValue(*c), ptr_delta + o));
                }

                BfInsn::SetAndMove(c, o) => {
                    cur_block.push(BfInsn::OffsetValue(ValueInsn::SetValue(*c), ptr_delta));

                    ptr_delta += o;
                }

                BfInsn::MovePtr(m) => {
                    ptr_delta += m;
                }

                it => panic!("Unexpected operation: {it:?}"),
            }
        }

        new_block.append(&mut cur_block);

        if ptr_delta != 0 {
            new_block.push(BfInsn::MovePtr(ptr_delta));
        }

        // extremely specific case where there's a single PointerMove followed by an offsettable
        // so for the case of something like `>>>-`
        // instead of [ValueChange(3, -1), PointerMove(3)]
        // we get [PointerMove(3), ValueChange(-1)]

        if new_block.len() == 2
            && new_block[0].offsettable()
            && let BfInsn::MovePtr(m) = new_block[1]
            && new_block[0].offset() == m
        {
            let first = new_block.remove(0);

            new_block.clear();
            new_block.push(BfInsn::MovePtr(m));

            new_block.push(match first {
                BfInsn::Value(v) => BfInsn::Value(v),
                BfInsn::OffsetValue(v, _) => BfInsn::Value(v),
                it => panic!("Unexpected operation: {it:?}"),
            });
        }

        let size = new_block.len();

        iter.swap(start..end, new_block);
        start += size + 1;
    }
}
