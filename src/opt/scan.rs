use crate::opt::{OptAction, Optimizer};

impl<'a> Optimizer<'a> {
    pub(super) fn scanners(&mut self) {
        let mut actions = Vec::new();

        std::mem::swap(&mut self.actions, &mut actions);

        for action in actions {
            if let OptAction::Loop(it) = action {
                if it.len() == 1
                    && let OptAction::MovePtr(m) = it[0]
                {
                    self.actions.push(OptAction::Scan(m));
                } else {
                    let mut opt = self.sub(it);

                    opt.scanners();

                    self.actions.push(OptAction::Loop(opt.actions));
                }
            } else {
                self.actions.push(action);
            }
        }
    }
}
