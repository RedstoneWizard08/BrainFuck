use crate::opt::{OptAction, Optimizer, ValueAction};

impl<'a> Optimizer<'a> {
    pub(super) fn simplify(&mut self) {
        let mut actions = Vec::new();

        std::mem::swap(&mut actions, &mut self.actions);

        for action in actions {
            match action {
                OptAction::Value(ValueAction::AddValue(0))
                | OptAction::MovePtr(0)
                | OptAction::Noop => (),

                OptAction::Loop(it) => {
                    let mut opt = self.sub(it);

                    opt.simplify();

                    self.actions.push(OptAction::Loop(opt.finish()));
                }

                other => self.actions.push(other),
            }
        }
    }
}
