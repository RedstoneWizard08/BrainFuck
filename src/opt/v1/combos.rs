use crate::{
    backend::Optimization,
    opt::{
        action::{OptAction, ValueAction},
        v1::Optimizer,
    },
};

impl<'a> Optimizer<'a> {
    pub(super) fn set_add(&mut self) {
        let mut actions = Vec::new();

        std::mem::swap(&mut self.actions, &mut actions);

        let mut buf = [OptAction::Noop, OptAction::Noop];

        for mut action in actions {
            buf.swap(0, 1);
            std::mem::swap(&mut buf[1], &mut action);

            if action != OptAction::Noop {
                self.actions.push(action);
            }

            match buf {
                [
                    OptAction::Value(ValueAction::SetValue(set)),
                    OptAction::Value(ValueAction::AddValue(add)),
                ] => {
                    self.actions
                        .push(OptAction::Value(ValueAction::SetValue(set + add)));
                    buf = [OptAction::Noop, OptAction::Noop];
                }

                [
                    OptAction::OffsetValue(ValueAction::SetValue(set), o1),
                    OptAction::OffsetValue(ValueAction::AddValue(add), o2),
                ] => {
                    if o1 == o2 {
                        self.actions
                            .push(OptAction::OffsetValue(ValueAction::SetValue(set + add), o1));
                        buf = [OptAction::Noop, OptAction::Noop];
                    }
                }

                [
                    OptAction::OffsetValue(ValueAction::SetValue(set), o1),
                    OptAction::MovePtr(o2),
                ] => {
                    if o1 == o2 {
                        self.actions.push(OptAction::MovePtr(o1));
                        self.actions
                            .push(OptAction::Value(ValueAction::SetValue(set)));

                        buf = [OptAction::Noop, OptAction::Noop];
                    }
                }

                _ => {}
            }
        }

        for action in buf {
            if action != OptAction::Noop {
                self.actions.push(action);
            }
        }

        self.optimize_loops(Optimization::SetAdd);
    }
}
