use crate::opt::{OptAction, Optimizer};

impl<'a> Optimizer<'a> {
    pub(super) fn offsets(&mut self) {
        let mut actions = Vec::new();

        std::mem::swap(&mut actions, &mut self.actions);

        for action in actions {
            if let OptAction::Loop(it) = action {
                let mut opt = self.sub(it);

                opt.offsets();

                self.actions.push(OptAction::Loop(opt.finish()));
            } else {
                if self.actions.len() < 2 {
                    self.actions.push(action);
                } else {
                    let a = &self.actions[self.actions.len() - 2];
                    let b = &self.actions[self.actions.len() - 1];

                    if let (OptAction::MovePtr(o), OptAction::Value(v)) = (a, b) {
                        let o = *o;
                        let v = *v;

                        if let OptAction::MovePtr(m) = action
                            && m == o
                        {
                            self.actions.pop();
                            self.actions.pop();
                            self.actions.push(OptAction::OffsetValue(v, o));
                        } else {
                            self.actions.push(action);
                        }
                    } else {
                        self.actions.push(action);
                    }
                }
            }
        }
    }
}
