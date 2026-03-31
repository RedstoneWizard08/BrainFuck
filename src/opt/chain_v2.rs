use crate::opt::base::{ActiveOptCx, BfInsn, ValueInsn};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ChainInsn {
    None,
    Add(i64),
    Set(i64),
    Move(i64),
    Print(i64),
}

impl ChainInsn {
    pub fn of(insn: BfInsn) -> Option<ChainInsn> {
        match insn {
            BfInsn::Value(ValueInsn::AddValue(v)) => Some(ChainInsn::Add(v)),
            BfInsn::Value(ValueInsn::SetValue(v)) => Some(ChainInsn::Set(v)),
            BfInsn::MovePtr(v) => Some(ChainInsn::Move(v)),
            BfInsn::Value(ValueInsn::Output) => Some(ChainInsn::Print(1)),
            _ => None,
        }
    }

    pub fn insn(&self) -> BfInsn {
        match self {
            Self::None => BfInsn::Noop,
            Self::Add(value) => BfInsn::Value(ValueInsn::AddValue(*value)),
            Self::Set(value) => BfInsn::Value(ValueInsn::SetValue(*value)),
            Self::Move(value) => BfInsn::MovePtr(*value),

            Self::Print(value) => {
                if *value == 1 {
                    BfInsn::Value(ValueInsn::Output)
                } else {
                    BfInsn::Value(ValueInsn::BulkPrint(*value))
                }
            }
        }
    }

    pub fn merge(&mut self, other: &ChainInsn) -> bool {
        match (self, other) {
            (ChainInsn::Add(me) | ChainInsn::Set(me), ChainInsn::Add(it)) => {
                *me = *me + *it;
                true
            }

            (ChainInsn::Set(me), ChainInsn::Set(it)) => {
                *me = *it;
                true
            }

            (ChainInsn::Move(me), ChainInsn::Move(it)) => {
                *me = *me + *it;
                true
            }

            (ChainInsn::Print(me), ChainInsn::Print(it)) => {
                *me = *me + *it;
                true
            }

            _ => false,
        }
    }
}

pub fn optimize_chains(mut cx: ActiveOptCx) {
    let iter = &mut cx.insns;
    let mut cur = ChainInsn::None;

    while let Some((insn, _)) = iter.next() {
        if let Some(it) = ChainInsn::of(insn) {
            if cur.merge(&it) {
                iter.remove_prev();
            } else {
                iter.replace_prev(cur.insn());
                cur = it;
            }
        } else {
            iter.insert_new(cur.insn());
            cur = ChainInsn::None;
        }
    }

    iter.insert(cur.insn());
}
