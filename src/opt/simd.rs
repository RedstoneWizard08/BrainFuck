use crate::opt::{OptAction, Optimizer, ValueAction};

impl<'a> Optimizer<'a> {
    pub(super) fn simd_add(&mut self) {
        let mut actions = Vec::new();

        std::mem::swap(&mut actions, &mut self.actions);

        let mut buf = Vec::new();
        let mut pos = 0;

        for insn in actions {
            if let OptAction::AddAndMove(a, m) = insn {
                if m < 0 {
                    if !buf.is_empty() {
                        self.actions.push(OptAction::SimdAddMove(buf, pos));
                        buf = Vec::new();
                        pos = 0;
                    }

                    self.actions.push(OptAction::AddAndMove(a, m));
                    continue;
                }

                let val = a as i8;

                while (buf.len() as i64) <= pos {
                    buf.push(0);
                }

                buf[pos as usize] = val;
                pos += m;
            } else {
                if !buf.is_empty() {
                    self.actions.push(OptAction::SimdAddMove(buf, pos));
                    buf = Vec::new();
                    pos = 0;
                }

                if let OptAction::Loop(it) = insn {
                    let mut opt = self.sub(it);

                    opt.simd_add();

                    self.actions.push(OptAction::Loop(opt.finish()));
                } else {
                    self.actions.push(insn);
                }
            }
        }

        if !buf.is_empty() {
            self.actions.push(OptAction::SimdAddMove(buf, pos));
        }

        self.simd_add_2();
    }

    pub(super) fn simd_add_2(&mut self) {
        let mut actions = Vec::new();

        std::mem::swap(&mut actions, &mut self.actions);

        let mut cur = Vec::new();
        let mut last = None;

        for action in actions {
            if let OptAction::OffsetValue(ValueAction::AddValue(a), o) = action {
                let needs_new = last.is_none_or(|it| it + 1 != o);

                if needs_new {
                    self.simd_add_finish_seq(&mut cur);
                }

                cur.push((a, o));
                last = Some(o);
            } else {
                self.simd_add_finish_seq(&mut cur);
                last = None;
                self.actions.push(action);
            }
        }

        self.simd_add_finish_seq(&mut cur);
    }

    fn simd_add_finish_seq(&mut self, cur: &mut Vec<(i64, i64)>) {
        if cur.is_empty() {
            return;
        }

        cur.sort_unstable_by_key(|it| it.1);

        let first = cur[0];
        let is_needed = cur.len() >= 16;

        if is_needed {
            self.actions.push(OptAction::MovePtr(first.1));

            // Do it this way (with the move and then move back) because otherwise it might
            // throw off expected positions, since we don't check for the move afterward.
            let len = cur.len() as i64;

            self.actions.push(OptAction::SimdAddMove(
                cur.iter().map(|it| it.0 as i8).collect(),
                len,
            ));

            self.actions.push(OptAction::MovePtr(-first.1 - len));
        } else {
            self.actions.extend(
                cur.iter()
                    .map(|it| OptAction::OffsetValue(ValueAction::AddValue(it.0), it.1)),
            );
        }

        cur.clear();
    }
}
