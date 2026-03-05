use crate::opt::{OptAction, Optimizer};

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
    }
}
