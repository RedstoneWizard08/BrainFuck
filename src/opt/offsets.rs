//! # Offset Adder
//!
//! Ported from: https://github.com/rhysdh540/bf/blob/main/src/commonMain/kotlin/dev/rdh/bf/opt/OffsetAdder.kt
//!
//! I understand how this works now that I've ported it, but huge thanks to rdh for
//! making an implementation I could base mine on. :)

use crate::opt::{OptAction, Optimizer, ValueAction};

trait Offsettable {
    fn offsettable(&self) -> bool;
    fn offset(&self) -> i64;
}

impl Offsettable for OptAction {
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

impl<'a> Optimizer<'a> {
    pub(super) fn offsets(&mut self) {
        let mut actions = Vec::new();

        std::mem::swap(&mut actions, &mut self.actions);

        let mut start = 0;

        while start < actions.len() {
            while start < actions.len() && !actions[start].offsettable() {
                self.actions.push(actions[start].clone());
                start += 1;
            }

            if start >= actions.len() {
                break;
            }

            let mut end = start;

            while end < actions.len() && actions[end].offsettable() {
                end += 1;
            }

            // the block [i..j) is a sequence of operations that can be replaced with offsets

            let mut new_block = Vec::new();
            let mut cur_block = Vec::new();
            let mut ptr_delta = 0;

            for pos in start..end {
                let op = &actions[pos];

                match op {
                    OptAction::Value(ValueAction::Input) => {
                        new_block.append(&mut cur_block);
                        new_block.push(OptAction::OffsetValue(ValueAction::Input, ptr_delta));
                    }

                    OptAction::OffsetValue(ValueAction::Input, o) => {
                        new_block.append(&mut cur_block);
                        new_block.push(OptAction::OffsetValue(ValueAction::Input, ptr_delta + o));
                    }

                    OptAction::Value(ValueAction::Output) => {
                        new_block.append(&mut cur_block);
                        new_block.push(OptAction::OffsetValue(ValueAction::Output, ptr_delta));
                    }

                    OptAction::OffsetValue(ValueAction::Output, o) => {
                        new_block.append(&mut cur_block);
                        new_block.push(OptAction::OffsetValue(ValueAction::Output, ptr_delta + o));
                    }

                    OptAction::Value(ValueAction::BulkPrint(n)) => {
                        new_block.append(&mut cur_block);

                        new_block.push(OptAction::OffsetValue(
                            ValueAction::BulkPrint(*n),
                            ptr_delta,
                        ));
                    }

                    OptAction::OffsetValue(ValueAction::BulkPrint(n), o) => {
                        new_block.extend(cur_block);
                        cur_block = Vec::new();

                        new_block.push(OptAction::OffsetValue(
                            ValueAction::BulkPrint(*n),
                            ptr_delta + o,
                        ));
                    }

                    OptAction::Value(ValueAction::AddValue(c)) => {
                        cur_block
                            .push(OptAction::OffsetValue(ValueAction::AddValue(*c), ptr_delta));
                    }

                    OptAction::OffsetValue(ValueAction::AddValue(c), o) => {
                        cur_block.push(OptAction::OffsetValue(
                            ValueAction::AddValue(*c),
                            ptr_delta + o,
                        ));
                    }

                    OptAction::AddAndMove(c, o) => {
                        cur_block
                            .push(OptAction::OffsetValue(ValueAction::AddValue(*c), ptr_delta));

                        ptr_delta += o;
                    }

                    OptAction::Value(ValueAction::SetValue(c)) => {
                        cur_block
                            .push(OptAction::OffsetValue(ValueAction::SetValue(*c), ptr_delta));
                    }

                    OptAction::OffsetValue(ValueAction::SetValue(c), o) => {
                        cur_block.push(OptAction::OffsetValue(
                            ValueAction::SetValue(*c),
                            ptr_delta + o,
                        ));
                    }

                    OptAction::SetAndMove(c, o) => {
                        cur_block
                            .push(OptAction::OffsetValue(ValueAction::SetValue(*c), ptr_delta));

                        ptr_delta += o;
                    }

                    OptAction::MovePtr(m) => {
                        ptr_delta += m;
                    }

                    it => panic!("Unexpected operation: {it:?}"),
                }
            }

            new_block.append(&mut cur_block);

            if ptr_delta != 0 {
                new_block.push(OptAction::MovePtr(ptr_delta));
            }

            // extremely specific case where there's a single PointerMove followed by an offsettable
            // so for the case of something like `>>>-`
            // instead of [ValueChange(3, -1), PointerMove(3)]
            // we get [PointerMove(3), ValueChange(-1)]

            if new_block.len() == 2
                && new_block[0].offsettable()
                && let OptAction::MovePtr(m) = new_block[1]
                && new_block[0].offset() == m
            {
                let first = new_block.remove(0);

                new_block.clear();
                new_block.push(OptAction::MovePtr(m));

                new_block.push(match first {
                    OptAction::Value(v) => OptAction::Value(v),
                    OptAction::OffsetValue(v, _) => OptAction::Value(v),
                    it => panic!("Unexpected operation: {it:?}"),
                });
            }

            start = end;
            self.actions.extend(new_block);
        }

        let opts = self.opts;

        for it in &mut self.actions {
            if let OptAction::Loop(lp) = it {
                let mut out = Vec::new();

                std::mem::swap(&mut out, lp);

                let mut opt = Optimizer {
                    opts,
                    actions: out,
                    depth: self.depth + 1,
                };

                opt.offsets();

                *lp = opt.actions;
            }
        }
    }
}
