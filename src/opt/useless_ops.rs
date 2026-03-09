use crate::opt::{OptAction, Optimizer, ValueAction};

impl<'a> Optimizer<'a> {
    pub(super) fn useless_ops(&mut self) {
        let mut actions = Vec::new();

        std::mem::swap(&mut actions, &mut self.actions);

        for action in actions {
            if action == OptAction::Value(ValueAction::SetValue(0)) {
                let mut cur = Vec::new();

                self.actions.reverse();

                std::mem::swap(&mut cur, &mut self.actions);

                let mut hit = false;

                for insn in cur {
                    if !insn.is_math() {
                        hit = true;
                    }

                    if hit {
                        self.actions.push(insn);
                    }
                }

                self.actions.reverse();
                self.actions.push(action);
            } else if let OptAction::SetAndMove(0, _) = action {
                let mut cur = Vec::new();

                self.actions.reverse();

                std::mem::swap(&mut cur, &mut self.actions);

                let mut hit = false;

                for insn in cur {
                    if !insn.is_math() {
                        hit = true;
                    }

                    if hit {
                        self.actions.push(insn);
                    }
                }

                self.actions.reverse();
                self.actions.push(action);
            } else if let OptAction::Loop(it) = action {
                let mut opt = self.sub(it);

                opt.useless_ops();

                self.actions.push(OptAction::Loop(opt.finish()));
            } else {
                self.actions.push(action);
            }
        }
    }
}
