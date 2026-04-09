use crate::opt::{
    action::{OptAction, ValueAction},
    v1::Optimizer,
};

impl<'a> Optimizer<'a> {
    pub(super) fn set_move(&mut self) {
        let mut actions = Vec::new();

        std::mem::swap(&mut actions, &mut self.actions);

        let mut prev: Option<OptAction> = None;

        for action in actions {
            if let OptAction::MovePtr(o) = action {
                if let Some(a) = prev {
                    if let OptAction::Value(ValueAction::SetValue(v)) = a {
                        self.actions.push(OptAction::SetAndMove(v, o));
                        prev = None;
                    } else if let OptAction::Value(ValueAction::AddValue(v)) = a {
                        self.actions.push(OptAction::AddAndMove(v, o));
                        prev = None;
                    } else if let OptAction::Loop(it) = a {
                        let mut opt = self.sub(it);

                        opt.set_move();

                        self.actions.push(OptAction::Loop(opt.finish()));
                        prev = Some(OptAction::MovePtr(o));
                    } else {
                        self.actions.push(a);
                        prev = Some(OptAction::MovePtr(o));
                    }
                } else {
                    prev = Some(OptAction::MovePtr(o));
                }
            } else if let Some(a) = prev {
                if let OptAction::Loop(it) = a {
                    let mut opt = self.sub(it);

                    opt.set_move();

                    self.actions.push(OptAction::Loop(opt.finish()));
                } else {
                    self.actions.push(a);
                }

                prev = Some(action);
            } else {
                prev = Some(action);
            }
        }

        if let Some(a) = prev {
            if let OptAction::Loop(it) = a {
                let mut opt = self.sub(it);

                opt.set_move();

                self.actions.push(OptAction::Loop(opt.finish()));
            } else {
                self.actions.push(a);
            }
        }
    }
}
