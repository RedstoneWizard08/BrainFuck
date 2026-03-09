use crate::opt::{OptAction, Optimizer, ValueAction};

impl<'a> Optimizer<'a> {
    pub(super) fn loops(&mut self) {
        let mut actions = Vec::new();

        std::mem::swap(&mut actions, &mut self.actions);

        let mut was_loop = false;

        for action in actions {
            match action {
                OptAction::Loop(it) => {
                    if was_loop {
                        continue;
                    }

                    was_loop = true;

                    if it.len() == 0 {
                        continue;
                    } else if it.len() == 1 && matches!(it[0], OptAction::Value(ValueAction::AddValue(_))) {
                        self.actions.push(OptAction::Value(ValueAction::SetValue(0)));
                    } else {
                        let mut opt = self.sub(it);

                        opt.loops();

                        self.actions.push(OptAction::Loop(opt.finish()));
                    }
                }

                other => {
                    was_loop = false;
                    self.actions.push(other);
                }
            }
        }
    }
}
