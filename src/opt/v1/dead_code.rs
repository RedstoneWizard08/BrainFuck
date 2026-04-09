use crate::opt::{
    action::{OptAction, ValueAction},
    v1::Optimizer,
};

impl<'a> Optimizer<'a> {
    pub(super) fn dead_code(&mut self) {
        self.dead_code_inner(true);
    }

    fn dead_code_inner(&mut self, start: bool) {
        let mut actions = Vec::new();

        std::mem::swap(&mut actions, &mut self.actions);

        let mut hit = !start;

        for action in actions {
            if !hit {
                if !matches!(action, OptAction::Loop(_)) {
                    hit = true;
                }
            }

            if hit {
                if let OptAction::CopyLoop(v) = action {
                    if v.len() != 0 {
                        self.actions.push(OptAction::CopyLoop(v));
                    } else {
                        self.actions
                            .push(OptAction::Value(ValueAction::SetValue(0)));
                    }
                } else if let OptAction::Loop(v) = action {
                    let mut opt = self.sub(v);

                    opt.dead_code_inner(false);

                    self.actions.push(OptAction::Loop(opt.finish()));
                } else {
                    self.actions.push(action);
                }
            }
        }
    }
}
